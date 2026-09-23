//! Legacy `/session` and experimental `/experimental/session` routes.
//!
//! These mirror the reference `session.*` and `experimental.session.*`
//! operations: raw arrays on list, NamedError bodies on failure, and the
//! `x-next-cursor`/`Link` pagination headers on the message list.

use crate::session::SessionInfo;
use crate::session_store::{
    CreateSession, GlobalListQuery, MessagePageError, SessionListQuery, SessionPatch,
};
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{OriginalUri, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use opencode_core::EventBus;
use opencode_schema::{EventEnvelope, EventType};
use serde::Deserialize;
use serde_json::{json, Value};

/// Routes for the legacy session group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/session", get(list).post(create))
        .route(
            "/session/{sessionID}",
            get(get_session).patch(update).delete(remove),
        )
        .route("/session/{sessionID}/abort", post(abort))
        .route("/session/{sessionID}/fork", post(fork))
        .route("/session/{sessionID}/children", get(children))
        .route("/session/{sessionID}/todo", get(todo))
        .route("/session/{sessionID}/message", get(messages))
        .route("/session/{sessionID}/message/{messageID}", get(message))
        .route("/session/{sessionID}/prompt", post(prompt))
        .route("/experimental/session", get(global_list))
        .route(
            "/experimental/session/{sessionID}/background",
            post(background),
        )
}

fn header_directory(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-opencode-directory")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn named_error(status: StatusCode, name: &str, message: String) -> Response {
    (
        status,
        Json(json!({ "name": name, "data": { "message": message } })),
    )
        .into_response()
}

fn session_not_found(id: &str) -> Response {
    named_error(
        StatusCode::NOT_FOUND,
        "NotFoundError",
        format!("Session not found: {id}"),
    )
}

fn message_not_found(id: &str) -> Response {
    named_error(
        StatusCode::NOT_FOUND,
        "NotFoundError",
        format!("Message not found: {id}"),
    )
}

fn bad_request(message: &str) -> Response {
    named_error(StatusCode::BAD_REQUEST, "BadRequest", message.to_string())
}

fn parse_bool(value: Option<String>) -> Option<bool> {
    value.map(|value| value == "true" || value == "1")
}

fn parse_i64(value: Option<String>) -> Option<i64> {
    value.and_then(|value| value.parse().ok())
}

fn parse_usize(value: Option<String>) -> Option<usize> {
    value.and_then(|value| value.parse().ok())
}

fn parse_archived(value: Option<&Value>) -> Option<i64> {
    value.and_then(|archived| {
        archived
            .as_i64()
            .or_else(|| archived.as_f64().map(|value| value as i64))
    })
}

fn create_from_value(value: &Value, directory: Option<String>) -> CreateSession {
    let string = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_string);
    CreateSession {
        title: string("title"),
        metadata: value
            .get("metadata")
            .cloned()
            .filter(|meta| !meta.is_null()),
        parent_id: string("parentID"),
        directory: string("directory").or(directory),
        agent: string("agent"),
        model: value.get("model").cloned().filter(|model| !model.is_null()),
    }
}

fn publish(state: &AppState, event_type: EventType, info: &SessionInfo) {
    state.bus.publish(EventEnvelope::new(
        event_type,
        json!({ "sessionID": info.id, "info": info }),
    ));
}

#[derive(Debug, Default, Deserialize)]
pub struct ListParams {
    #[serde(default)]
    directory: Option<String>,
    #[serde(default)]
    roots: Option<String>,
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    limit: Option<String>,
}

/// `GET /session`.
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
    headers: HeaderMap,
) -> Response {
    let directory = if params.directory.is_some() {
        params
            .directory
            .clone()
            .or_else(|| header_directory(&headers))
    } else {
        None
    };
    let query = SessionListQuery {
        directory,
        roots: parse_bool(params.roots),
        start: parse_i64(params.start),
        search: params.search,
        limit: parse_usize(params.limit),
        order: None,
    };
    Json(state.sessions.list(&query)).into_response()
}

/// `POST /session`.
pub async fn create(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let directory = header_directory(&headers);
    let input = if body.is_empty() {
        CreateSession {
            directory,
            ..CreateSession::default()
        }
    } else {
        match serde_json::from_slice::<Value>(&body) {
            Ok(value) => create_from_value(&value, directory),
            Err(_) => return bad_request("Invalid request body"),
        }
    };
    let info = state.sessions.create(input);
    publish(&state, EventType::SessionCreated, &info);
    Json(info).into_response()
}

/// `GET /session/{sessionID}`.
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Response {
    match state.sessions.get(&session_id) {
        Some(info) => Json(info).into_response(),
        None => session_not_found(&session_id),
    }
}

/// `PATCH /session/{sessionID}`.
pub async fn update(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    body: Bytes,
) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return bad_request("Invalid request body"),
    };
    let patch = SessionPatch {
        title: value
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        metadata: value
            .get("metadata")
            .cloned()
            .filter(|metadata| !metadata.is_null()),
        permission: value
            .get("permission")
            .cloned()
            .filter(|permission| !permission.is_null()),
        archived: parse_archived(value.get("time").and_then(|time| time.get("archived"))),
    };
    match state.sessions.update(&session_id, patch) {
        Some(info) => {
            publish(&state, EventType::SessionUpdated, &info);
            Json(info).into_response()
        }
        None => session_not_found(&session_id),
    }
}

/// `DELETE /session/{sessionID}`.
pub async fn remove(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.remove(&session_id) {
        Json(true).into_response()
    } else {
        session_not_found(&session_id)
    }
}

/// `POST /session/{sessionID}/fork`.
pub async fn fork(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    body: Bytes,
) -> Response {
    if !body.is_empty() && serde_json::from_slice::<Value>(&body).is_err() {
        return bad_request("Invalid request body");
    }
    match state.sessions.fork(&session_id, None) {
        Some(info) => {
            publish(&state, EventType::SessionCreated, &info);
            Json(info).into_response()
        }
        None => session_not_found(&session_id),
    }
}

/// `POST /session/{sessionID}/abort`.
pub async fn abort() -> Json<bool> {
    Json(true)
}

/// `GET /session/{sessionID}/children`.
pub async fn children(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    let all = state.sessions.list(&SessionListQuery::default());
    let children: Vec<SessionInfo> = all
        .into_iter()
        .filter(|info| info.parent_id.as_deref() == Some(session_id.as_str()))
        .collect();
    Json(children).into_response()
}

/// `GET /session/{sessionID}/todo`.
pub async fn todo(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    Json(Vec::<Value>::new()).into_response()
}

#[derive(Debug, Default, Deserialize)]
pub struct MessagesParams {
    #[serde(default)]
    limit: Option<String>,
    #[serde(default)]
    before: Option<String>,
}

/// `GET /session/{sessionID}/message`.
pub async fn messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<MessagesParams>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    if params.before.is_some() && params.limit.is_none() {
        return bad_request("A cursor requires a limit");
    }
    let limit = match params.limit.as_deref() {
        None => None,
        Some(raw) => match raw.parse::<usize>() {
            Ok(limit) => Some(limit),
            Err(_) => return bad_request("Invalid limit"),
        },
    };
    let page = match state
        .sessions
        .messages(&session_id, limit, params.before.as_deref())
    {
        Ok(page) => page,
        Err(MessagePageError::SessionNotFound) => return session_not_found(&session_id),
        Err(MessagePageError::InvalidCursor) => return bad_request("Invalid cursor"),
    };
    let mut response = Json(Value::Array(page.items)).into_response();
    if let Some(cursor) = page.cursor {
        let link = format!(
            "<{}?limit={}&before={cursor}>; rel=\"next\"",
            uri.path(),
            limit.unwrap_or(0)
        );
        let headers = response.headers_mut();
        headers.insert(
            "access-control-expose-headers",
            HeaderValue::from_static("Link, X-Next-Cursor"),
        );
        if let Ok(value) = HeaderValue::from_str(&cursor) {
            headers.insert("x-next-cursor", value);
        }
        if let Ok(value) = HeaderValue::from_str(&link) {
            headers.insert("link", value);
        }
    }
    response
}

/// `GET /session/{sessionID}/message/{messageID}`.
pub async fn message(
    State(state): State<AppState>,
    Path((session_id, message_id)): Path<(String, String)>,
) -> Response {
    match state.sessions.message(&session_id, &message_id) {
        Some(message) => Json(message).into_response(),
        None => message_not_found(&message_id),
    }
}

/// `POST /session/{sessionID}/prompt`.
pub async fn prompt(State(state): State<AppState>, Path(session_id): Path<String>) -> Response {
    if state.sessions.get(&session_id).is_none() {
        return session_not_found(&session_id);
    }
    named_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "ServiceUnavailableError",
        "Session prompt is not available yet".to_string(),
    )
}

#[derive(Debug, Default, Deserialize)]
pub struct GlobalListParams {
    #[serde(default)]
    directory: Option<String>,
    #[serde(default)]
    roots: Option<String>,
    #[serde(default)]
    start: Option<String>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    limit: Option<String>,
    #[serde(default)]
    archived: Option<String>,
}

/// `GET /experimental/session`.
pub async fn global_list(
    State(state): State<AppState>,
    Query(params): Query<GlobalListParams>,
    headers: HeaderMap,
) -> Response {
    let directory = params
        .directory
        .clone()
        .or_else(|| header_directory(&headers));
    let query = GlobalListQuery {
        directory,
        roots: parse_bool(params.roots),
        start: parse_i64(params.start),
        search: params.search,
        limit: parse_usize(params.limit),
        order: None,
        archived: parse_bool(params.archived).unwrap_or(false),
        cursor: parse_i64(params.cursor),
    };
    Json(state.sessions.list_global(&query)).into_response()
}

/// `POST /experimental/session/{sessionID}/background`.
pub async fn background() -> Json<bool> {
    Json(false)
}
