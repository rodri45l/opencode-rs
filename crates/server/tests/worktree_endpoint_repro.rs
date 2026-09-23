//! Port of packages/opencode/test/server/worktree-endpoint-repro.test.ts (upstream 18ef3cc).
//!
//! Subset: the experimental worktree create route accepts an omitted body and
//! rejects an explicit `null` payload, and the workspace worktree create route
//! does not hang. The full lifecycle assertions need a live git worktree and
//! the worktree ready event; those are left to the workspace phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-worktree-endpoint-repro-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn post(uri: &str) -> Request<Body> {
    with_directory(request("POST", uri).body(Body::empty()).unwrap())
}

fn post_raw(uri: &str, body: &str) -> Request<Body> {
    with_directory(
        request("POST", uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
}

#[tokio::test]
#[ignore = "porting: worktree routes not implemented"]
async fn direct_worktree_create_returns_without_waiting_for_boot() {
    let app = router(AppState::new());
    let res = send(
        &app,
        post(&format!("/experimental/worktree?directory={DIRECTORY}")),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await["directory"].is_string());
}

#[tokio::test]
#[ignore = "porting: worktree routes not implemented"]
async fn direct_worktree_create_accepts_missing_content_type_and_body() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(
            request(
                "POST",
                &format!("/experimental/worktree?directory={DIRECTORY}"),
            )
            .body(Body::empty())
            .unwrap(),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await["directory"].is_string());
}

#[tokio::test]
#[ignore = "porting: worktree routes not implemented"]
async fn direct_worktree_create_rejects_explicit_null_payload() {
    let app = router(AppState::new());
    let res = send(
        &app,
        post_raw(
            &format!("/experimental/worktree?directory={DIRECTORY}"),
            "null",
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "porting: workspace routes not implemented"]
async fn workspace_worktree_create_does_not_hang() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(common::json_body(
            request(
                "POST",
                &format!("/experimental/workspace?directory={DIRECTORY}"),
            )
            .body(Body::empty())
            .unwrap(),
            &serde_json::json!({ "type": "worktree", "branch": null }),
        )),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert_eq!(body["type"], "worktree");
    assert!(body["directory"].is_string());
}
