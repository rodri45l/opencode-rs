//! Port of packages/opencode/test/server/httpapi-mcp.test.ts (upstream 18ef3cc).
//!
//! Subset: the MCP status/add/connect/disconnect surface and the typed
//! not-found errors for missing servers. Cases that assert a specific
//! configured server's disabled status depend on the config loader; the port
//! asserts the same wire shapes against the router.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-mcp-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn req(method: &str, uri: &str) -> Request<Body> {
    with_directory(request(method, uri).body(Body::empty()).unwrap())
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn serves_status_endpoint() {
    let app = router(AppState::new());
    let res = send(&app, req("GET", "/mcp")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await,
        serde_json::json!({ "demo": { "status": "disabled" } })
    );
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn serves_add_connect_and_disconnect_endpoints() {
    let app = router(AppState::new());

    let added = send(
        &app,
        with_directory(json_body(
            request("POST", "/mcp").body(Body::empty()).unwrap(),
            &serde_json::json!({
                "name": "added",
                "config": { "type": "local", "command": ["echo", "added"], "enabled": false },
            }),
        )),
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    assert_eq!(json(added).await["added"]["status"], "disabled");

    let disconnected = send(&app, req("POST", "/mcp/added/disconnect")).await;
    assert_eq!(disconnected.status(), StatusCode::OK);
    assert_eq!(json(disconnected).await, serde_json::json!(true));

    let connected = send(&app, req("POST", "/mcp/demo/connect")).await;
    assert_eq!(connected.status(), StatusCode::OK);
    assert_eq!(json(connected).await, serde_json::json!(true));

    let disconnected = send(&app, req("POST", "/mcp/demo/disconnect")).await;
    assert_eq!(disconnected.status(), StatusCode::OK);
    assert_eq!(json(disconnected).await, serde_json::json!(true));
}

#[tokio::test]
#[ignore = "porting: mcp routes not implemented"]
async fn returns_typed_not_found_errors_for_missing_mcp_servers() {
    let app = router(AppState::new());

    for (method, route, body) in [
        ("POST", "/mcp/missing/auth", None),
        ("POST", "/mcp/missing/auth/authenticate", None),
        (
            "POST",
            "/mcp/missing/auth/callback",
            Some(r#"{"code":"code"}"#),
        ),
        ("DELETE", "/mcp/missing/auth", None),
        ("POST", "/mcp/missing/connect", None),
        ("POST", "/mcp/missing/disconnect", None),
    ] {
        let request = match body {
            Some(body) => with_directory(
                request(method, route)
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            ),
            None => req(method, route),
        };
        let res = send(&app, request).await;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            json(res).await,
            serde_json::json!({
                "_tag": "McpServerNotFoundError",
                "name": "missing",
                "message": "MCP server not found: missing",
            })
        );
    }
}
