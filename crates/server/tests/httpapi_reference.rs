//! Port of packages/opencode/test/server/httpapi-reference.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `/api/reference` route; see docs/TEST-PORT.md.
//!
//! The reference waits for the reference registry to resolve; the Rust port
//! asserts the resolved shape directly against the router.

mod common;

use axum::body::Body;
use axum::http::StatusCode;
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-reference-port";

#[tokio::test]
async fn lists_usable_references_resolved_in_the_server_workspace() {
    let app = router(AppState::new());
    let req = common::header(
        request("GET", "/api/reference")
            .body(Body::empty())
            .unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;

    assert_eq!(body["location"]["directory"], DIRECTORY);
    assert_eq!(
        body["data"],
        serde_json::json!([
            {
                "name": "docs",
                "path": format!("{DIRECTORY}/docs"),
                "source": { "type": "local", "path": format!("{DIRECTORY}/docs") },
            },
            {
                "name": "effect",
                "path": format!("{DIRECTORY}/repos/github.com/Effect-TS/effect@main"),
                "source": {
                    "type": "git",
                    "repository": "Effect-TS/effect",
                    "branch": "main",
                },
            },
        ])
    );
}
