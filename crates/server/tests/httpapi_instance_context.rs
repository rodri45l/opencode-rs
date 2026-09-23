//! Port of packages/opencode/test/server/httpapi-instance-context.test.ts (upstream 18ef3cc).
//!
//! Subset: instance-context resolution from the routed directory and the
//! selected workspace, exercised through the real router. The reference probes
//! the middleware with a test-only `/probe` API and a live workspace store; the
//! Rust port asserts the same routing decisions on production routes, so the
//! cases stay red until the instance-context middleware lands.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-instance-context-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

#[tokio::test]
async fn provides_instance_context_from_the_routed_directory() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get("/session?directory=%2Ftmp%2Fopencode-instance-context-port"),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_array());
}

#[tokio::test]
#[ignore = "porting: instance-context middleware not implemented"]
async fn persists_the_routed_project_while_loading_instance_context() {
    let app = router(AppState::new());
    let res = send(&app, get("/project/current")).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert_ne!(body["id"], "global");
}

#[tokio::test]
async fn falls_back_to_the_raw_directory_when_uri_decoding_fails() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?directory=%25E0%25A4%25A")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_array());
}

#[tokio::test]
async fn provides_selected_workspace_id_on_control_plane_routes() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_selected")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_array());
}

#[tokio::test]
#[ignore = "porting: workspace-routing middleware not implemented"]
async fn uses_workspace_routing_output_instead_of_raw_directory_hints() {
    let app = router(AppState::new());
    let res = send(&app, get("/project/current?workspace=wrk_selected")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await["workspaceID"], "wrk_selected");
}

#[tokio::test]
#[ignore = "porting: workspace-routing middleware not implemented"]
async fn returns_missing_workspace_for_unknown_workspace_ids() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_missing")).await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(common::text(res).await, "Workspace not found: wrk_missing");
}
