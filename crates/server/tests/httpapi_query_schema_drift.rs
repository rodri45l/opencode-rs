//! Port of packages/opencode/test/server/httpapi-query-schema-drift.test.ts (upstream 18ef3cc).
//!
//! Subset: every route that advertises `directory`/`workspace` routing query
//! params must accept them at runtime (no 400). The reference asserts `not 400`
//! (the route may legitimately return another status); the port asserts `200`
//! so the red-first suite stays red until the route is actually mounted. The
//! reference also asserts the generated OpenAPI metadata; that is covered by
//! the OpenAPI port.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-query-schema-drift-port";

fn get(uri: &str) -> Request<Body> {
    common::header(
        request("GET", uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    )
}

fn routing_params() -> String {
    format!("directory={DIRECTORY}&workspace=ws_test")
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn session_list_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(&app, get(&format!("/session?{}", routing_params()))).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn session_messages_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!(
            "/session/ses_abc/message?limit=80&{}",
            routing_params()
        )),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn file_find_file_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!("/find/file?query=foo&{}", routing_params())),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn file_find_text_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!("/find?pattern=foo&{}", routing_params())),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn file_read_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(&app, get(&format!("/file?path=foo&{}", routing_params()))).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: experimental session routes not implemented"]
async fn experimental_session_list_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!("/experimental/session?{}", routing_params())),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: experimental tool routes not implemented"]
async fn experimental_tool_list_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!(
            "/experimental/tool?provider=anthropic&model=claude&{}",
            routing_params()
        )),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: vcs routes not implemented"]
async fn vcs_diff_accepts_directory_and_workspace() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!("/vcs/diff?mode=working&{}", routing_params())),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}
