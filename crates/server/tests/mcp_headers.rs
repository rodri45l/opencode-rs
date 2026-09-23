//! Port of packages/opencode/test/mcp/headers.test.ts (upstream 18ef3cc).
//!
//! Subset: adding a remote MCP server with configured headers reports it as
//! connected, and adding one without headers still connects. The reference also
//! asserts the upstream transport received the configured headers; that needs
//! the MCP runtime and a live transport and is left to the MCP phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-mcp-headers-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn add_server(value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("POST", "/mcp").body(Body::empty()).unwrap(),
        value,
    ))
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn headers_are_passed_to_transports_when_oauth_is_enabled() {
    let app = router(AppState::new());
    let res = send(
        &app,
        add_server(&serde_json::json!({
            "name": "test-server",
            "config": {
                "type": "remote",
                "url": "http://127.0.0.1:1/mcp",
                "headers": {
                    "Authorization": "Bearer test-token",
                    "X-Custom-Header": "custom-value",
                },
            },
        })),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await["test-server"]["status"], "connected");
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn headers_are_passed_to_transports_when_oauth_is_explicitly_disabled() {
    let app = router(AppState::new());
    let res = send(
        &app,
        add_server(&serde_json::json!({
            "name": "test-server-no-oauth",
            "config": {
                "type": "remote",
                "url": "http://127.0.0.1:1/mcp",
                "oauth": false,
                "headers": { "Authorization": "Bearer test-token" },
            },
        })),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await["test-server-no-oauth"]["status"],
        "connected"
    );
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn connects_without_request_init_when_headers_are_not_provided() {
    let app = router(AppState::new());
    let res = send(
        &app,
        add_server(&serde_json::json!({
            "name": "test-server-no-headers",
            "config": { "type": "remote", "url": "http://127.0.0.1:1/mcp" },
        })),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await["test-server-no-headers"]["status"],
        "connected"
    );
}
