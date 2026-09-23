//! Port of packages/opencode/test/server/httpapi-mcp-oauth.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the MCP OAuth start route; see docs/TEST-PORT.md.
//!
//! Re-derived: the reference wires a mocked MCP handler that returns a fixed
//! authorization URL; the Rust port asserts the same response shape from the
//! real router.

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-mcp-oauth-port";

#[tokio::test]
async fn preserves_oauth_state_when_starting_oauth() {
    let app = router(AppState::new());
    let req = common::header(
        request("POST", "/mcp/demo/auth")
            .body(Body::empty())
            .unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "authorizationUrl": "https://auth.example/start",
            "oauthState": "state-123",
        })
    );
}
