//! v2 `/api/session` routes.
//!
//! These mirror the reference `v2.session.*` operations: `{ data }` envelopes,
//! `{ data, cursor }` list envelopes, and the tagged `ApiError` bodies.

use crate::error::ApiErrorResponse;
use crate::session::{SessionMessagesResponse, SessionsResponse};
use crate::session_store::{CreateSession, SessionListQuery};
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use opencode_core::EventBus;
use opencode_protocol::{
    decode_cursor, encode_cursor, AnchorDirection, ApiError, Cursor, ListAnchor, Order,
    SessionsCursorInput,
};
use opencode_schema::{EventEnvelope, EventType, SessionId};
use serde::Deserialize;
use serde_json::{json, Value};

/// Routes for the v2 session group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/api/session", get(list).post(create))
        .route("/api/session/{sessionID}", get(get_session))
        .route("/api/session/{sessionID}/message", get(messages))
        .route("/api/session/{sessionID}/message/{messageID}", get(message))
        .route("/api/session/{sessionID}/history", get(history))
        .route("/api/session/{sessionID}/context", get(context))
        .route("/api/session/{sessionID}/compact", post(compact))
        .route("/api/session/{sessionID}/wait", post(wait))
        .route("/api/session/{sessionID}/prompt", post(prompt))
}

fn header_directory(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-opencode-directory")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn api_error(error: ApiError) -> Response {
    ApiErrorResponse(error).into_response()
}

fn session_not_found(id: &str) -> Response {
    api_error(ApiError::SessionNotFound {
        session_id: id.to_string(),
        message: format!("Session not found: {id}"),
    })
}

fn invalid_request(message: &str, kind: &str, field: &str) -> Response {
    api_error(ApiError::InvalidRequest {
        message: message.to_string(),
        kind: Some(kind.to_string()),
        field: Some(field.to_string()),
    })
}

fn unavailable(message: &str, service: &str) -> Response {
    api_error(ApiError::ServiceUnavailable {
        message: message.to_string(),
        service: Some(service.to_string()),
    })
}

fn parse_order(value: Option<&str>) -> Order {
    match value {
        Some("asc") => Order::Asc,
        _ => Order::Desc,
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
    #[serde(default)]
    limit: Option<String>,
    #[serde(default)]
    order: Option<String>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    directory: Option<String>,
    #[serde(default)]
    cursor: Option<String>,
}

/// `GET /api/session`.
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
    headers: HeaderMap,
) -> Response {
    let limit = match params.limit.as_deref() {
        None => 50usize,
        Some(raw) => match raw.parse::<i64>() {
            Ok(value) if value > 0 => value as usize,
            _ => {
                return invalid_request("limit must be a positive integer", "Query", "limit");
            }
        },
    };

    let order = parse_order(params.order.as_deref());
    let directory = params
        .directory
        .clone()
        .or_else(|| header_directory(&headers));

    let anchor = match params.cursor.as_deref() {
        None => None,
        Some(raw) => match decode_cursor(raw) {
            Ok(input) => Some(input.anchor),
            Err(_) => {
                return api_error(ApiError::InvalidCursor {
                    message: "Invalid cursor".to_string(),
                });
            }
        },
    };

    let query = SessionListQuery {
        directory,
        roots: None,
        start: None,
        search: params.search.clone(),
        limit: None,
        order: Some(order),
    };
    let mut items = state.sessions.list(&query);
    if let Some(anchor) = &anchor {
        match order {
            Order::Desc => {
                items.retain(|info| info.time.updated.unwrap_or(info.time.created) < anchor.time)
            }
            Order::Asc => {
                items.retain(|info| info.time.updated.unwrap_or(info.time.created) > anchor.time)
            }
        }
    }

    let more = items.len() > limit;
    if more {
        items.truncate(limit);
    }
    let next = if more {
        items
            .last()
            .and_then(|info| anchor_input(info, order, AnchorDirection::Next))
    } else {
        None
    };
    let previous = anchor
        .as_ref()
        .and_then(|_| items.first())
        .and_then(|info| anchor_input(info, order, AnchorDirection::Previous));

    Json(SessionsResponse {
        data: items,
        cursor: Cursor { previous, next },
    })
    .into_response()
}

fn anchor_input(
    info: &crate::session::SessionInfo,
    order: Order,
    direction: AnchorDirection,
) -> Option<String> {
    let anchor = SessionsCursorInput {
        workspace: None,
        order: Some(order),
        search: None,
        directory: Some(info.directory.clone()),
        project: None,
        subpath: None,
        anchor: ListAnchor {
            id: SessionId::parse(info.id.clone()).ok()?,
            time: info.time.updated.unwrap_or(info.time.created),
            direction,
        },
    };
    encode_cursor(&anchor).ok()
}

/// `POST /api/session`.
pub async fn create(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let directory = header_directory(&headers);
    let input = if body.is_empty() {
        CreateSession {
            directory,
            ..CreateSession::default()
        }
    } else {
        match serde_json::from_slice::<Value>(&body) {
            Ok(value) => CreateSession {
                directory: value
                    .get("directory")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(directory),
                agent: value
                    .get("agent")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                model: value.get("model").cloned().filter(|model| !model.is_null()),
                ..CreateSession::default()
            },
            Err(_) => return invalid_request("Invalid request body", "Body", "body"),
        }
    };
    let info = state.sessions.create(input);
    state.bus.publish(EventEnvelope::new(
        EventType::SessionCreated,
        json!({ "sessionID": info.id, "info": info }),
    ));
    Json(json!({ "data": info })).into_response()
}

/// `GET /api/session/{sessionID}`.
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Response {
    match state.sessions.get(&session_id) {
        Some(info) => Json(json!({ "data": info })).into_response(),
        None => session_not_found(&session_id),
    }
}

/// `GET /api/session/{sessionID}/message`.
pub async fn messages(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    let page = match state.sessions.messages(&session_id, None, None) {
        Ok(page) => page,
        Err(_) => return session_not_found(&session_id),
    };
    Json(SessionMessagesResponse {
        data: page.items,
        cursor: Cursor {
            previous: None,
            next: page.cursor,
        },
    })
    .into_response()
}

/// `GET /api/session/{sessionID}/message/{messageID}`.
pub async fn message(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    match state.sessions.message(&session_id, &message_id) {
        Some(message) => Json(json!({ "data": message })).into_response(),
        None => api_error(ApiError::MessageNotFound {
            session_id: session_id.clone(),
            message_id: message_id.clone(),
            message: format!("Message not found: {message_id}"),
        }),
    }
}

/// `GET /api/session/{sessionID}/history`.
pub async fn history(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    match state.sessions.history(&session_id) {
        Some(history) => Json(history).into_response(),
        None => session_not_found(&session_id),
    }
}

/// `GET /api/session/{sessionID}/context`.
pub async fn context(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    match state.sessions.context(&session_id) {
        Some(messages) => Json(json!({ "data": messages })).into_response(),
        None => session_not_found(&session_id),
    }
}

/// `POST /api/session/{sessionID}/compact`.
pub async fn compact(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    unavailable("Session compact is not available yet", "session.compact")
}

/// `POST /api/session/{sessionID}/wait`.
pub async fn wait(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    unavailable("Session wait is not available yet", "session.wait")
}

/// `POST /api/session/{sessionID}/prompt`.
pub async fn prompt(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    unavailable("Session prompt is not available yet", "session.prompt")
}
