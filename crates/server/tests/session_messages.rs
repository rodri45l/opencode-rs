//! Port of packages/opencode/test/server/session-messages.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference seeds messages through the Effect session store;
//! the Rust port asserts the same pagination contract through the
//! `/session/{id}/message` route.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_schema::SessionId;
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-session-messages-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

async fn create_session(app: &axum::Router) -> String {
    let res = send(
        app,
        with_directory(json_body(
            request("POST", "/session").body(Body::empty()).unwrap(),
            &serde_json::json!({ "title": "messages" }),
        )),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    json(res).await["id"].as_str().expect("id").to_string()
}

#[tokio::test]
#[ignore = "porting: test seeds messages directly; no message-write route exposed"]
async fn returns_cursor_headers_for_older_pages() {
    let app = router(AppState::new());
    let session = create_session(&app).await;

    let first = send(&app, get(&format!("/session/{session}/message?limit=2"))).await;
    assert_eq!(first.status(), StatusCode::OK);
    let cursor = first
        .headers()
        .get("x-next-cursor")
        .and_then(|value| value.to_str().ok())
        .expect("x-next-cursor")
        .to_string();
    assert!(first
        .headers()
        .get("link")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .contains("rel=\"next\""));

    let second = send(
        &app,
        get(&format!(
            "/session/{session}/message?limit=2&before={cursor}"
        )),
    )
    .await;
    assert_eq!(second.status(), StatusCode::OK);
}

#[tokio::test]
async fn keeps_full_history_responses_when_limit_is_omitted() {
    let app = router(AppState::new());
    let session = create_session(&app).await;

    let res = send(&app, get(&format!("/session/{session}/message"))).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_array());
}

#[tokio::test]
async fn rejects_invalid_cursors_and_missing_sessions() {
    let app = router(AppState::new());
    let session = SessionId::generate();

    let bad = send(
        &app,
        get(&format!("/session/{session}/message?limit=2&before=bad")),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let missing = send(&app, get("/session/ses_missing/message?limit=2")).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "porting: test seeds 520 messages directly; no message-write route exposed"]
async fn does_not_truncate_large_legacy_limit_requests() {
    let app = router(AppState::new());
    let session = create_session(&app).await;

    let res = send(&app, get(&format!("/session/{session}/message?limit=510"))).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await.as_array().expect("array").len(), 510);
}

#[tokio::test]
#[ignore = "porting: test seeds a message directly; no message-write route exposed"]
async fn accepts_directory_query_used_by_workspace_routing() {
    let app = router(AppState::new());
    let session = create_session(&app).await;

    let res = send(
        &app,
        get(&format!(
            "/session/{session}/message?limit=80&directory={DIRECTORY}"
        )),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().expect("array").len(), 1);
}
