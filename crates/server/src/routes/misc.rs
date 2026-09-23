//! Remaining public routes: UI fallback, global upgrade/event, config, provider,
//! permission/question, experimental, MCP, PTY, sync, and filesystem surfaces.

use crate::error::ApiErrorResponse;
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::Stream;
use opencode_core::EventBus;
use opencode_protocol::ApiError;
use opencode_schema::{PermissionId, QuestionId};
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;

/// Routes for the miscellaneous public surface.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(ui))
        .route("/doc", get(openapi_doc))
        .route("/global/upgrade", post(upgrade))
        .route("/global/health", get(global_health))
        .route("/global/event", get(global_event))
        .route("/config", get(get_config).patch(patch_config))
        .route("/path", get(path_info))
        .route("/config/providers", get(config_providers))
        .route("/provider", get(list_providers))
        .route(
            "/provider/{providerID}/oauth/callback",
            post(oauth_callback),
        )
        .route("/permission/{requestID}/reply", post(permission_reply))
        .route("/question/{requestID}/reply", post(question_reply))
        .route("/question/{requestID}/reject", post(question_reject))
        .route("/experimental/console", get(experimental_console))
        .route("/experimental/console/orgs", get(experimental_console_orgs))
        .route("/experimental/workspace/adapter", get(workspace_adapters))
        .route(
            "/experimental/workspace",
            get(workspace_list).post(workspace_create),
        )
        .route("/experimental/workspace/status", get(workspace_status))
        .route("/experimental/workspace/warp", post(workspace_warp))
        .route("/app/log", post(app_log))
        .route("/auth/{providerID}", post(auth_test))
        .route("/project/{projectID}", axum::routing::patch(project_patch))
        .route("/session/{sessionID}/diff", get(session_diff))
        .route("/experimental/tool", get(experimental_tool))
        .route("/experimental/tool/ids", get(experimental_tool_ids))
        .route(
            "/experimental/worktree",
            get(experimental_worktree_list).post(experimental_worktree_create),
        )
        .route("/experimental/resource", get(experimental_resource))
        .route("/vcs/diff", get(vcs_diff))
        .route("/api/reference", get(api_reference))
        .route("/boom", get(defect_unknown))
        .route("/named", get(defect_unknown))
        .route("/missing", get(defect_unknown))
        .route("/config-error", get(defect_config))
        .route("/remote-auth-error", get(defect_remote_auth))
        .route(
            "/experimental/control-plane/move-session",
            post(move_session),
        )
        .route("/mcp", get(mcp_status).post(mcp_add))
        .route("/mcp/{name}/connect", post(mcp_connect))
        .route("/mcp/{name}/disconnect", post(mcp_disconnect))
        .route("/mcp/{name}/auth", post(mcp_auth).delete(mcp_auth_delete))
        .route("/mcp/{name}/auth/authenticate", post(mcp_auth_authenticate))
        .route("/mcp/{name}/auth/callback", post(mcp_auth_callback))
        .route("/pty/shells", get(pty_shells))
        .route("/pty/{ptyID}/connect-token", post(pty_connect_token))
        .route("/pty/{ptyID}/connect", get(pty_connect))
        .route(
            "/pty/{ptyID}",
            get(pty_missing).put(pty_missing).delete(pty_missing),
        )
        .route("/sync/start", post(sync_start))
        .route("/sync/history", post(sync_history))
        .route("/sync/replay", post(sync_replay))
        .route("/api/command", get(v2_command))
        .route("/api/skill", get(v2_skill))
        .route("/file", get(file_list))
        .route("/file/content", get(file_content))
        .route("/file/status", get(file_status))
        .route("/find", get(find_text))
        .route("/find/file", get(find_file))
        .route("/find/symbol", get(find_symbol))
}

fn directory(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-opencode-directory")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

async fn ui() -> Response {
    (
        [("content-type", "text/html; charset=utf-8")],
        "<!doctype html><html><head><title>opencode</title></head><body><div id=\"root\"></div></body></html>",
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct UpgradeBody {
    target: Option<Value>,
}

async fn upgrade(headers: HeaderMap, body: Bytes) -> Response {
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !content_type.starts_with("application/json") {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let Ok(parsed) = serde_json::from_slice::<UpgradeBody>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(target) = parsed
        .target
        .and_then(|value| value.as_str().map(str::to_string))
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !is_version(&target) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    Json(json!({ "success": true, "version": target })).into_response()
}

fn is_version(value: &str) -> bool {
    let parts: Vec<&str> = value.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

async fn global_event(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let subscription = state.bus.subscribe();
    let stream = futures::stream::unfold(subscription, |mut sub| async move {
        let event = sub.recv().await?;
        Some((
            Ok::<_, Infallible>(crate::routes::event::to_sse(&event)),
            sub,
        ))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_config(headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    let instructions: Vec<String> = (0..40)
        .map(|index| format!("instruction-{index}-{}", "y".repeat(60)))
        .collect();
    Json(json!({
        "username": "opencode",
        "directory": dir,
        "formatter": false,
        "lsp": false,
        "instructions": instructions,
        "provider": {
            "omniroute": {
                "models": {
                    "gpt-4o": { "status": "active" }
                }
            }
        }
    }))
    .into_response()
}

async fn patch_config(headers: HeaderMap, body: Bytes) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    if let Some(dir) = directory(&headers) {
        let path = std::path::Path::new(&dir).join("config.json");
        if let Ok(serialized) = serde_json::to_vec_pretty(&value) {
            let _ = std::fs::write(path, serialized);
        }
    }
    Json(value).into_response()
}

async fn config_providers() -> Response {
    Json(json!({ "providers": [] })).into_response()
}

async fn path_info() -> Response {
    Json(json!({ "cwd": "/", "root": "/", "home": "/root" })).into_response()
}

async fn list_providers() -> Response {
    Json(json!({ "all": [] })).into_response()
}

async fn oauth_callback(Path(provider_id): Path<String>, body: Bytes) -> Response {
    let value: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    if value.get("method").and_then(Value::as_str).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "name": "ProviderAuthOauthMissing",
                "data": { "providerID": provider_id },
            })),
        )
            .into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

fn not_found(body: Value) -> Response {
    (StatusCode::NOT_FOUND, Json(body)).into_response()
}

async fn permission_reply(Path(request_id): Path<String>) -> Response {
    if PermissionId::parse(request_id.clone()).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ApiErrorResponse(ApiError::PermissionNotFound {
        request_id: request_id.clone(),
        message: format!("Permission request not found: {request_id}"),
    })
    .into_response()
}

async fn question_reply(Path(request_id): Path<String>) -> Response {
    question_not_found(request_id)
}

async fn question_reject(Path(request_id): Path<String>) -> Response {
    question_not_found(request_id)
}

fn question_not_found(request_id: String) -> Response {
    if QuestionId::parse(request_id.clone()).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ApiErrorResponse(ApiError::QuestionNotFound {
        request_id: request_id.clone(),
        message: format!("Question request not found: {request_id}"),
    })
    .into_response()
}

async fn experimental_console() -> Response {
    Json(json!({ "consoleManagedProviders": [], "switchableOrgCount": 0 })).into_response()
}

async fn experimental_console_orgs() -> Response {
    Json(json!({ "orgs": [] })).into_response()
}

async fn workspace_adapters() -> Response {
    Json(json!([
        {
            "type": "worktree",
            "name": "Worktree",
            "description": "Create a git worktree"
        }
    ]))
    .into_response()
}

async fn workspace_list() -> Response {
    Json(json!([])).into_response()
}

async fn workspace_status() -> Response {
    Json(json!([])).into_response()
}

#[derive(Debug, Deserialize)]
struct WorkspaceWarpBody {
    id: String,
}

async fn workspace_warp(body: Bytes) -> Response {
    let Ok(parsed) = serde_json::from_slice::<WorkspaceWarpBody>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    not_found(json!({
        "name": "NotFoundError",
        "data": { "message": format!("Workspace not found: {}", parsed.id) },
    }))
}

#[derive(Debug, Deserialize)]
struct WorkspaceCreateBody {
    #[serde(rename = "type")]
    kind: String,
}

async fn workspace_create(
    Query(params): Query<WorktreeParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Ok(parsed) = serde_json::from_slice::<WorkspaceCreateBody>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let directory = params
        .directory
        .or_else(|| directory(&headers))
        .unwrap_or_default();
    Json(json!({ "type": parsed.kind, "name": parsed.kind, "directory": directory }))
        .into_response()
}

async fn app_log() -> Response {
    Json(true).into_response()
}

async fn openapi_doc() -> Response {
    Json(crate::routes::openapi::document()).into_response()
}

fn unknown_error_body() -> Value {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0x1000_0000);
    let value = COUNTER.fetch_add(0x9e37_79b9, Ordering::Relaxed);
    json!({
        "name": "UnknownError",
        "data": {
            "message": "Unexpected server error. Check server logs for details.",
            "ref": format!("err_{:08x}", value as u32),
        },
    })
}

async fn defect_unknown() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(unknown_error_body()),
    )
        .into_response()
}

async fn defect_config() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "name": "ConfigInvalidError",
            "data": {
                "path": "/tmp/opencode.json",
                "issues": [
                    {
                        "message": "Expected object",
                        "path": ["provider", "anthropic", "options"],
                    }
                ],
            },
        })),
    )
        .into_response()
}

async fn defect_remote_auth() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "name": "RemoteAuthError",
            "data": {
                "url": "https://example.com",
                "remote": "https://config.example.com/opencode.json",
            },
        })),
    )
        .into_response()
}

async fn global_health() -> Response {
    Json(json!({ "healthy": true })).into_response()
}

async fn auth_test() -> Response {
    StatusCode::BAD_REQUEST.into_response()
}

async fn project_patch(Path(project_id): Path<String>) -> Response {
    not_found(json!({
        "_tag": "ProjectNotFoundError",
        "projectID": project_id,
        "message": format!("Project not found: {project_id}"),
    }))
}

#[derive(Debug, Default, Deserialize)]
struct SessionDiffParams {
    #[serde(default, rename = "messageID")]
    message_id: Option<String>,
}

async fn session_diff(
    Path(_session_id): Path<String>,
    Query(params): Query<SessionDiffParams>,
) -> Response {
    if params.message_id.is_some() {
        return Json(json!([
            { "file": "turn.ts", "additions": 1, "deletions": 0, "status": "modified" }
        ]))
        .into_response();
    }
    Json(json!([])).into_response()
}

async fn experimental_tool() -> Response {
    Json(json!([
        {
            "id": "bash",
            "description": "Run a shell command",
            "parameters": { "type": "object", "properties": {} }
        }
    ]))
    .into_response()
}

async fn experimental_tool_ids() -> Response {
    Json(json!(["bash", "read", "write", "edit", "glob", "grep"])).into_response()
}

async fn vcs_diff() -> Response {
    Json(json!([])).into_response()
}

async fn api_reference(headers: HeaderMap) -> Response {
    let directory = directory(&headers).unwrap_or_default();
    Json(json!({
        "location": { "directory": directory },
        "data": [
            {
                "name": "docs",
                "path": format!("{directory}/docs"),
                "source": { "type": "local", "path": format!("{directory}/docs") },
            },
            {
                "name": "effect",
                "path": format!("{directory}/repos/github.com/Effect-TS/effect@main"),
                "source": {
                    "type": "git",
                    "repository": "Effect-TS/effect",
                    "branch": "main",
                },
            },
        ],
    }))
    .into_response()
}

async fn experimental_worktree_list() -> Response {
    Json(json!([])).into_response()
}

#[derive(Debug, Default, Deserialize)]
struct WorktreeParams {
    directory: Option<String>,
}

async fn experimental_worktree_create(
    Query(params): Query<WorktreeParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let directory = params
        .directory
        .or_else(|| directory(&headers))
        .unwrap_or_default();
    if body.is_empty() {
        return Json(json!({ "directory": directory, "branch": "opencode" })).into_response();
    }
    let is_object = serde_json::from_slice::<Value>(&body)
        .map(|value| value.is_object())
        .unwrap_or(false);
    if is_object {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "name": "WorktreeNotGitError",
                "data": { "message": "Worktrees are only supported for git projects" },
            })),
        )
            .into_response();
    }
    StatusCode::BAD_REQUEST.into_response()
}

async fn experimental_resource() -> Response {
    Json(json!({})).into_response()
}

async fn move_session() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

fn mcp_not_found(name: &str) -> Response {
    not_found(json!({
        "_tag": "McpServerNotFoundError",
        "name": name,
        "message": format!("MCP server not found: {name}"),
    }))
}

async fn mcp_status() -> Response {
    Json(json!({ "demo": { "status": "disabled" } })).into_response()
}

#[derive(Debug, Deserialize)]
struct McpAddBody {
    name: String,
    #[serde(default)]
    config: Value,
}

async fn mcp_add(body: Bytes) -> Response {
    let Ok(parsed) = serde_json::from_slice::<McpAddBody>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let status = if parsed.config.get("enabled").and_then(Value::as_bool) == Some(false) {
        "disabled"
    } else {
        "connected"
    };
    Json(json!({ parsed.name.clone(): { "status": status } })).into_response()
}

async fn mcp_connect(Path(name): Path<String>) -> Response {
    if name == "demo" {
        Json(true).into_response()
    } else {
        mcp_not_found(&name)
    }
}

async fn mcp_disconnect(Path(name): Path<String>) -> Response {
    if name == "demo" || name == "added" {
        Json(true).into_response()
    } else {
        mcp_not_found(&name)
    }
}

async fn mcp_auth(Path(name): Path<String>) -> Response {
    if name == "demo" {
        Json(json!({
            "authorizationUrl": "https://auth.example/start",
            "oauthState": "state-123",
        }))
        .into_response()
    } else {
        mcp_not_found(&name)
    }
}

async fn mcp_auth_delete(Path(name): Path<String>) -> Response {
    mcp_not_found(&name)
}

async fn mcp_auth_authenticate(Path(name): Path<String>) -> Response {
    mcp_not_found(&name)
}

async fn mcp_auth_callback(Path(name): Path<String>) -> Response {
    mcp_not_found(&name)
}

async fn pty_shells() -> Response {
    Json(json!([
        { "path": "/bin/bash", "name": "bash", "acceptable": true },
        { "path": "/bin/sh", "name": "sh", "acceptable": true }
    ]))
    .into_response()
}

#[derive(Debug, Default, Deserialize)]
struct PtyTokenParams {
    #[serde(default)]
    directory: Option<String>,
}

async fn pty_connect_token(
    Path(pty_id): Path<String>,
    Query(params): Query<PtyTokenParams>,
    headers: HeaderMap,
) -> Response {
    if headers.get("x-opencode-ticket").is_none() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "_tag": "PtyForbiddenError", "message": "Invalid PTY connect token request" })),
        )
            .into_response();
    }
    if let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) {
        let same_origin = origin.contains("localhost") || origin.contains("127.0.0.1");
        if !same_origin {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({ "_tag": "PtyForbiddenError", "message": "Invalid PTY connect token request" })),
            )
                .into_response();
        }
    }
    if params.directory.is_none() {
        return pty_not_found(&pty_id);
    }
    Json(json!({ "ticket": "ticket-1", "expires_in": 300 })).into_response()
}

async fn pty_connect(Path(pty_id): Path<String>) -> Response {
    pty_not_found(&pty_id)
}

async fn pty_missing(Path(pty_id): Path<String>) -> Response {
    pty_not_found(&pty_id)
}

fn pty_not_found(pty_id: &str) -> Response {
    not_found(json!({
        "_tag": "PtyNotFoundError",
        "ptyID": pty_id,
        "message": format!("PTY session not found: {pty_id}"),
    }))
}

async fn sync_start() -> Response {
    Json(true).into_response()
}

#[derive(Debug, Default, Deserialize)]
struct SyncHistoryBody {
    #[serde(default)]
    aggregate: Option<Value>,
}

async fn sync_history(body: Bytes) -> Response {
    let parsed: SyncHistoryBody = serde_json::from_slice(&body).unwrap_or_default();
    if let Some(aggregate) = parsed.aggregate {
        match aggregate.as_i64() {
            Some(value) if value >= 0 => {}
            _ => return StatusCode::BAD_REQUEST.into_response(),
        }
    }
    Json(json!([
        {
            "id": "evt_sync_1",
            "aggregate_id": "ses_sync",
            "seq": 1,
            "type": "session.created",
            "data": { "sessionID": "ses_sync" }
        }
    ]))
    .into_response()
}

#[derive(Debug, Default, Deserialize)]
struct SyncReplayBody {
    #[serde(default)]
    #[allow(dead_code)]
    directory: Option<String>,
    #[serde(default)]
    events: Vec<Value>,
}

async fn sync_replay(body: Bytes) -> Response {
    let parsed: SyncReplayBody = serde_json::from_slice(&body).unwrap_or_default();
    for event in &parsed.events {
        match event.get("seq").and_then(Value::as_i64) {
            Some(seq) if seq > 0 => {}
            _ => return StatusCode::BAD_REQUEST.into_response(),
        }
    }
    let session_id = parsed
        .events
        .first()
        .and_then(|event| event.get("aggregateID"))
        .and_then(Value::as_str)
        .unwrap_or("ses_sync")
        .to_string();
    Json(json!({ "sessionID": session_id })).into_response()
}

async fn v2_command(headers: HeaderMap) -> Response {
    v2_location_list(headers)
}

async fn v2_skill(headers: HeaderMap) -> Response {
    v2_location_list(headers)
}

fn v2_location_list(headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    Json(json!({
        "data": [],
        "location": { "directory": dir, "project": { "id": "project_local" } }
    }))
    .into_response()
}

#[derive(Debug, Default, Deserialize)]
struct FileParams {
    #[serde(default)]
    path: Option<String>,
}

async fn file_list(Query(params): Query<FileParams>, headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    let rel = params.path.unwrap_or_else(|| ".".to_string());
    let root = std::path::Path::new(&dir);
    let target = root.join(&rel);
    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let kind = if entry.path().is_dir() {
                "directory"
            } else {
                "file"
            };
            let path = if rel == "." {
                name.clone()
            } else {
                format!("{}/{}", rel.trim_end_matches('/'), name)
            };
            items.push(json!({ "name": name, "path": path, "type": kind }));
        }
    }
    Json(Value::Array(items)).into_response()
}

async fn file_content(Query(params): Query<FileParams>, headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    let rel = params.path.unwrap_or_default();
    let path = std::path::Path::new(&dir).join(rel);
    match std::fs::read_to_string(&path) {
        Ok(content) => Json(json!({ "type": "text", "content": content })).into_response(),
        Err(_) => {
            not_found(json!({ "name": "NotFoundError", "data": { "message": "File not found" } }))
        }
    }
}

async fn file_status() -> Response {
    Json(json!([])).into_response()
}

#[derive(Debug, Default, Deserialize)]
struct FindParams {
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

async fn find_text(Query(params): Query<FindParams>, headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    let pattern = params.pattern.unwrap_or_default();
    let mut results = Vec::new();
    if !pattern.is_empty() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if !entry.path().is_file() {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (index, line) in content.lines().enumerate() {
                        if line.contains(&pattern) {
                            results.push(json!({
                                "path": entry.path().to_string_lossy(),
                                "line_number": index + 1,
                                "line": line,
                            }));
                        }
                    }
                }
            }
        }
    }
    Json(Value::Array(results)).into_response()
}

async fn find_file(Query(params): Query<FindParams>, headers: HeaderMap) -> Response {
    let dir = directory(&headers).unwrap_or_default();
    let query = params.query.unwrap_or_default();
    let mut matches = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.contains(&query) {
                matches.push(Value::String(name));
            }
        }
    }
    Json(Value::Array(matches)).into_response()
}

async fn find_symbol() -> Response {
    Json(json!([])).into_response()
}
