//! Port of packages/opencode/test/server/httpapi-control-plane.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the root control-plane move-session route; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json_body, request, send};
use opencode_server::{router, AppState};

#[tokio::test]
async fn moves_a_session_through_the_root_control_plane_route() {
    let app = router(AppState::new());
    let req = json_body(
        request("POST", "/experimental/control-plane/move-session")
            .body(Body::empty())
            .unwrap(),
        &serde_json::json!({
            "sessionID": "ses_move",
            "destination": { "directory": "/destination" },
            "moveChanges": true,
        }),
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
