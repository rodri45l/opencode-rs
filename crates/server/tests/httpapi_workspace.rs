//! Port of packages/opencode/test/server/httpapi-workspace.test.ts (upstream 18ef3cc).
//!
//! Subset: the workspace read endpoints, the declared not-found error when
//! warping into a missing workspace, and the TUI create payload shape. Cases
//! that register workspace adapters, create real git worktrees, or proxy to a
//! remote target need the control-plane and are left to the workspace phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-workspace-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

fn post_json(uri: &str, value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("POST", uri).body(Body::empty()).unwrap(),
        value,
    ))
}

#[tokio::test]
async fn serves_read_endpoints() {
    let app = router(AppState::new());

    let adapters = send(&app, get("/experimental/workspace/adapter")).await;
    assert_eq!(adapters.status(), StatusCode::OK);
    assert!(json(adapters)
        .await
        .as_array()
        .expect("array")
        .iter()
        .any(|item| item["type"] == "worktree"
            && item["name"] == "Worktree"
            && item["description"] == "Create a git worktree"));

    let workspaces = send(&app, get("/experimental/workspace")).await;
    assert_eq!(workspaces.status(), StatusCode::OK);
    assert_eq!(json(workspaces).await, serde_json::json!([]));

    let status = send(&app, get("/experimental/workspace/status")).await;
    assert_eq!(status.status(), StatusCode::OK);
    assert_eq!(json(status).await, serde_json::json!([]));
}

#[tokio::test]
async fn returns_a_declared_not_found_error_when_warping_into_a_missing_workspace() {
    let app = router(AppState::new());
    let workspace_id = "wrk_missing_warp";

    let res = send(
        &app,
        post_json(
            "/experimental/workspace/warp",
            &serde_json::json!({ "id": workspace_id, "sessionID": "ses_abc" }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "name": "NotFoundError",
            "data": { "message": format!("Workspace not found: {workspace_id}") },
        })
    );
}

#[tokio::test]
async fn creates_workspace_with_the_tui_payload_shape() {
    let app = router(AppState::new());
    let res = send(
        &app,
        post_json(
            "/experimental/workspace",
            &serde_json::json!({ "type": "local-test", "branch": null }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert_eq!(body["type"], "local-test");
    assert_eq!(body["name"], "local-test");
}
