//! Port of packages/opencode/test/server/httpapi-pty.test.ts (upstream 18ef3cc).
//!
//! Subset: the non-WebSocket PTY contract — shell listing, missing-socket
//! 404s, typed not-found PTY resources, and connect-token failures. The
//! WebSocket echo and lifecycle cases require a real PTY bridge and are left to
//! the PTY phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_schema::PtyId;
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-pty-port";

fn req(method: &str, uri: &str) -> Request<Body> {
    common::header(
        request(method, uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    )
}

#[tokio::test]
#[ignore = "porting: pty routes not implemented"]
async fn serves_available_shell_list() {
    let app = router(AppState::new());
    let res = send(&app, req("GET", "/pty/shells")).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    let shells = body.as_array().expect("shell array");
    assert!(!shells.is_empty());
    assert!(shells.iter().any(|shell| {
        shell["path"].is_string() && shell["name"].is_string() && shell["acceptable"].is_boolean()
    }));
}

#[tokio::test]
#[ignore = "porting: pty routes not implemented"]
async fn returns_404_for_missing_pty_websocket_before_upgrade() {
    let app = router(AppState::new());
    // Precondition: the PTY surface is mounted (otherwise the connect 404 is
    // indistinguishable from the router's unknown-path fallback).
    assert_eq!(
        send(&app, req("GET", "/pty/shells")).await.status(),
        StatusCode::OK
    );
    let id = PtyId::generate();
    let res = send(&app, req("GET", &format!("/pty/{id}/connect"))).await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "porting: pty routes not implemented"]
async fn returns_404_for_missing_pty_websocket_before_decoding_cursor_query() {
    let app = router(AppState::new());
    assert_eq!(
        send(&app, req("GET", "/pty/shells")).await.status(),
        StatusCode::OK
    );
    let id = PtyId::generate();
    let res = send(
        &app,
        req("GET", &format!("/pty/{id}/connect?cursor=a&cursor=b")),
    )
    .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "porting: pty routes not implemented"]
async fn returns_typed_not_found_errors_for_missing_pty_http_resources() {
    let app = router(AppState::new());
    let id = PtyId::generate();
    let expected = serde_json::json!({
        "_tag": "PtyNotFoundError",
        "ptyID": id.to_string(),
        "message": format!("PTY session not found: {id}"),
    });

    let found = send(&app, req("GET", &format!("/pty/{id}"))).await;
    assert_eq!(found.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(found).await, expected);

    let updated = send(&app, req("PUT", &format!("/pty/{id}"))).await;
    assert_eq!(updated.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(updated).await, expected);

    let removed = send(&app, req("DELETE", &format!("/pty/{id}"))).await;
    assert_eq!(removed.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(removed).await, expected);
}

#[tokio::test]
#[ignore = "porting: pty connect-token route not implemented"]
async fn returns_typed_errors_for_pty_connect_token_failures() {
    let app = router(AppState::new());
    let id = PtyId::generate();

    let forbidden = send(&app, req("POST", &format!("/pty/{id}/connect-token"))).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        json(forbidden).await,
        serde_json::json!({
            "_tag": "PtyForbiddenError",
            "message": "Invalid PTY connect token request",
        })
    );

    let req = common::header(
        req("POST", &format!("/pty/{id}/connect-token")),
        "x-opencode-ticket",
        "1",
    );
    let missing = send(&app, req).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(missing).await,
        serde_json::json!({
            "_tag": "PtyNotFoundError",
            "ptyID": id.to_string(),
            "message": format!("PTY session not found: {id}"),
        })
    );
}
