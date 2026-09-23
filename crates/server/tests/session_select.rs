//! Port of packages/opencode/test/server/session-select.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `tui.selectSession` route; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-session-select-port";

fn select(session_id: &str) -> axum::http::Request<Body> {
    let req = json_body(
        request("POST", "/tui/select-session")
            .body(Body::empty())
            .unwrap(),
        &serde_json::json!({ "sessionID": session_id }),
    );
    common::header(req, "x-opencode-directory", DIRECTORY)
}

#[tokio::test]
#[ignore = "porting: tui.selectSession route not implemented"]
async fn returns_200_when_called_with_valid_session() {
    let app = router(AppState::new());
    let res = send(&app, select("ses_valid")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await, serde_json::json!(true));
}

#[tokio::test]
#[ignore = "porting: tui.selectSession route not implemented"]
async fn returns_404_when_session_does_not_exist() {
    let app = router(AppState::new());
    let res = send(&app, select("ses_nonexistent123")).await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    // The endpoint's own not-found is a JSON error, not the router fallback.
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap_or_default()
        .contains("application/json"));
}

#[tokio::test]
#[ignore = "porting: tui.selectSession route not implemented"]
async fn returns_400_when_session_id_format_is_invalid() {
    let app = router(AppState::new());
    let res = send(&app, select("invalid_session_id")).await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
