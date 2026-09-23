//! Port of packages/opencode/test/server/httpapi-workspace-routing.test.ts (upstream 18ef3cc).
//!
//! Subset: the workspace-routing middleware contract — proxy a selected remote
//! workspace request (HTTP and WebSocket), surface the sync fence, report an
//! inactive sync as `503`, report an unknown workspace as `500`, keep
//! control-plane routes local, and fall back to directory hints. The reference
//! stands up fake remote servers and a live workspace store; the Rust port
//! drives the real router, so the cases stay red until the middleware lands.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-workspace-routing-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

fn patch(uri: &str) -> Request<Body> {
    with_directory(request("PATCH", uri).body(Body::empty()).unwrap())
}

#[tokio::test]
#[ignore = "porting: workspace-routing middleware not implemented"]
async fn proxies_remote_workspace_http_requests_through_the_selected_target() {
    let app = router(AppState::new());
    let res = send(
        &app,
        patch("/session/ses_remote?workspace=wrk_remote&keep=yes"),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    assert_eq!(res.headers()["x-remote"], "yes");
    let body = common::json(res).await;
    assert_eq!(body["proxied"], true);
    assert_eq!(body["keep"], "yes");
    assert_eq!(body["workspace"], serde_json::Value::Null);
}

#[tokio::test]
#[ignore = "porting: workspace-routing sync fence not implemented"]
async fn waits_for_sync_fence_headers_from_remote_workspace_responses() {
    let app = router(AppState::new());
    let res = send(&app, patch("/session/ses_fence?workspace=wrk_fence")).await;

    assert_eq!(res.status(), StatusCode::ACCEPTED);
    assert_eq!(
        common::json(res).await,
        serde_json::json!({ "proxied": true })
    );
}

#[tokio::test]
#[ignore = "porting: workspace-routing middleware not implemented"]
async fn returns_503_when_a_remote_workspace_is_not_actively_syncing() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_not_syncing")).await;

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        common::text(res).await,
        "broken sync connection for workspace: wrk_not_syncing"
    );
}

#[tokio::test]
#[ignore = "porting: workspace-routing websocket proxy not implemented"]
async fn proxies_remote_workspace_websocket_requests_through_the_selected_target() {
    let app = router(AppState::new());
    let req = common::header(
        get("/session?workspace=wrk_ws"),
        "sec-websocket-protocol",
        "chat",
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
#[ignore = "porting: workspace-routing middleware not implemented"]
async fn returns_a_missing_workspace_response_for_unknown_workspace_ids() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_missing")).await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(common::text(res).await, "Workspace not found: wrk_missing");
}

#[tokio::test]
async fn keeps_control_plane_routes_local_even_when_workspace_is_selected() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_local")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(common::json(res).await.is_array());
}

#[tokio::test]
async fn keeps_workspace_control_routes_local_even_when_workspace_is_selected() {
    let app = router(AppState::new());
    let res = send(&app, get("/experimental/workspace?workspace=wrk_local")).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn uses_directory_query_and_header_fallback_when_no_workspace_is_selected() {
    let app = router(AppState::new());

    let query = send(
        &app,
        with_directory(get("/session?directory=%2Ftmp%2Fquery-target")),
    )
    .await;
    assert_eq!(query.status(), StatusCode::OK);

    let header = send(
        &app,
        common::header(
            request("GET", "/session").body(Body::empty()).unwrap(),
            "x-opencode-directory",
            "/tmp/header-target",
        ),
    )
    .await;
    assert_eq!(header.status(), StatusCode::OK);
}

#[tokio::test]
async fn routes_local_workspace_requests_through_the_workspace_route_context() {
    let app = router(AppState::new());
    let res = send(&app, get("/session?workspace=wrk_local_target")).await;

    assert_eq!(res.status(), StatusCode::OK);
}
