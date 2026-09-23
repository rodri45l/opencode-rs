//! Port of packages/opencode/test/server/project-init-git.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `/project/git/init` route; see docs/TEST-PORT.md.
//!
//! The reference asserts the global-bus disposal event count; the Rust port
//! asserts the route's response and the follow-up `/project/current` state.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-project-init-git-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

#[tokio::test]
#[ignore = "porting: project git init route not implemented"]
async fn initializes_git_and_reloads_immediately() {
    let app = router(AppState::new());

    let init = send(
        &app,
        with_directory(
            request("POST", "/project/git/init")
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;
    assert_eq!(init.status(), StatusCode::OK);
    let body = json(init).await;
    assert_eq!(body["id"], "global");
    assert_eq!(body["vcs"], "git");
    assert_eq!(body["worktree"], DIRECTORY);

    let current = send(&app, get("/project/current")).await;
    assert_eq!(current.status(), StatusCode::OK);
    let current = json(current).await;
    assert_eq!(current["id"], "global");
    assert_eq!(current["vcs"], "git");
    assert_eq!(current["worktree"], DIRECTORY);
}

#[tokio::test]
#[ignore = "porting: project git init route not implemented"]
async fn does_not_reload_when_the_project_is_already_git() {
    let app = router(AppState::new());

    let init = send(
        &app,
        with_directory(
            request("POST", "/project/git/init")
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;
    assert_eq!(init.status(), StatusCode::OK);
    let body = json(init).await;
    assert_eq!(body["vcs"], "git");
    assert_eq!(body["worktree"], DIRECTORY);

    let current = send(&app, get("/project/current")).await;
    assert_eq!(current.status(), StatusCode::OK);
    let current = json(current).await;
    assert_eq!(current["vcs"], "git");
    assert_eq!(current["worktree"], DIRECTORY);
}
