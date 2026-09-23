//! Port of packages/opencode/test/server/session-diff-missing-patch.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `GET /session/:id/diff`; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-session-diff-port";

fn get(uri: &str) -> axum::http::Request<Body> {
    common::header(
        request("GET", uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    )
}

#[tokio::test]
async fn ignores_legacy_session_level_diff_storage() {
    let app = router(AppState::new());
    let res = send(&app, get("/session/ses_missing_patch/diff")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await, serde_json::json!([]));
}

#[tokio::test]
async fn returns_requested_turn_diffs() {
    let app = router(AppState::new());
    let res = send(&app, get("/session/ses_turn_diff/diff?messageID=msg_turn")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await,
        serde_json::json!([{ "file": "turn.ts", "additions": 1, "deletions": 0, "status": "modified" }])
    );
}
