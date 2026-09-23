//! Port of packages/opencode/test/server/sdk-v1-smoke.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference drives the v1 SDK against the server; the Rust
//! port asserts the same reachable core endpoints and wire shapes directly.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-sdk-v1-smoke-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn session_list_reaches_the_server_and_returns_an_array() {
    let app = router(AppState::new());
    let res = send(&app, get("/session")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_array());
}

#[tokio::test]
#[ignore = "porting: path routes not implemented"]
async fn path_get_reaches_the_server() {
    let app = router(AppState::new());
    let res = send(&app, get("/path")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_object());
}

#[tokio::test]
#[ignore = "porting: config routes not implemented"]
async fn config_get_reaches_the_server() {
    let app = router(AppState::new());
    let res = send(&app, get("/config")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.is_object());
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn session_404_result_tuple_path_returns_the_error_body() {
    let app = router(AppState::new());
    let res = send(&app, get("/session/ses_no_such")).await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(res).await["name"], "NotFoundError");
}
