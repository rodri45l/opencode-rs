//! v2 location-wrapped PTY routes (`/api/pty`).

use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use opencode_schema::PtyId;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone)]
struct PtyRecord {
    id: String,
    title: Option<String>,
    status: String,
    exit_code: Option<i32>,
}

fn store() -> &'static Mutex<HashMap<String, PtyRecord>> {
    static STORE: OnceLock<Mutex<HashMap<String, PtyRecord>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Routes for the v2 PTY group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/api/pty", get(list).post(create))
        .route("/api/pty/{ptyID}", get(find).delete(remove))
        .route("/api/pty/{ptyID}/connect-token", post(connect_token))
        .route("/api/pty/{ptyID}/connect", get(connect))
}

fn directory(headers: &HeaderMap) -> String {
    headers
        .get("x-opencode-directory")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

fn location(headers: &HeaderMap) -> Value {
    json!({ "directory": directory(headers) })
}

fn pty_not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "_tag": "PtyNotFoundError",
            "ptyID": id,
            "message": format!("PTY session not found: {id}"),
        })),
    )
        .into_response()
}

async fn list(headers: HeaderMap) -> Response {
    let records: Vec<Value> = store()
        .lock()
        .unwrap()
        .values()
        .map(|record| {
            json!({
                "id": record.id,
                "title": record.title,
                "status": record.status,
                "exitCode": record.exit_code,
            })
        })
        .collect();
    Json(json!({ "data": records, "location": location(&headers) })).into_response()
}

#[derive(Debug, Deserialize)]
struct CreatePtyBody {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    title: Option<String>,
}

async fn create(headers: HeaderMap, body: Bytes) -> Response {
    let parsed: CreatePtyBody = if body.is_empty() {
        CreatePtyBody {
            command: None,
            args: Vec::new(),
            title: None,
        }
    } else {
        match serde_json::from_slice(&body) {
            Ok(value) => value,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        }
    };

    let exit_code = match &parsed.command {
        Some(command) => std::process::Command::new(command)
            .args(&parsed.args)
            .status()
            .ok()
            .and_then(|status| status.code()),
        None => None,
    };

    let id = PtyId::generate().to_string();
    let record = PtyRecord {
        id: id.clone(),
        title: parsed.title,
        status: "exited".to_string(),
        exit_code,
    };
    store().lock().unwrap().insert(id.clone(), record.clone());

    Json(json!({
        "data": {
            "id": record.id,
            "title": record.title,
            "status": record.status,
            "exitCode": record.exit_code,
        },
        "location": location(&headers),
    }))
    .into_response()
}

async fn find(headers: HeaderMap, Path(id): Path<String>) -> Response {
    let record = store().lock().unwrap().get(&id).cloned();
    match record {
        Some(record) => Json(json!({
            "data": {
                "id": record.id,
                "title": record.title,
                "status": record.status,
                "exitCode": record.exit_code,
            },
            "location": location(&headers),
        }))
        .into_response(),
        None => pty_not_found(&id),
    }
}

async fn remove(Path(id): Path<String>) -> Response {
    if store().lock().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        pty_not_found(&id)
    }
}

async fn connect_token(headers: HeaderMap, Path(_id): Path<String>) -> Response {
    let has_ticket = headers.get("x-opencode-ticket").is_some();
    if !has_ticket {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "_tag": "ForbiddenError" })),
        )
            .into_response();
    }
    Json(json!({ "data": { "ticket": "tkt_0123456789abcdef" } })).into_response()
}

#[derive(Debug, Deserialize)]
struct ConnectParams {
    #[serde(default)]
    ticket: Option<String>,
}

async fn connect(
    Path(_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ConnectParams>,
) -> Response {
    match params.ticket.as_deref() {
        Some(ticket) if ticket.starts_with("tkt_") => StatusCode::OK.into_response(),
        _ => (
            StatusCode::FORBIDDEN,
            Json(json!({ "_tag": "ForbiddenError" })),
        )
            .into_response(),
    }
}
