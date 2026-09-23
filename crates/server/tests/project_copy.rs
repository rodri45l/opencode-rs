//! Port of packages/opencode/test/server/project-copy.test.ts (upstream 18ef3cc).
//!
//! Subset: listing project directories and the git-worktree copy lifecycle
//! (generate-name, create, forced remove, refresh). The reference relies on a
//! live git worktree and snapshot service; the Rust port asserts the same HTTP
//! contract through the router.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-project-copy-port";

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

fn delete_json(uri: &str, value: &serde_json::Value) -> Request<Body> {
    with_directory(json_body(
        request("DELETE", uri).body(Body::empty()).unwrap(),
        value,
    ))
}

#[tokio::test]
#[ignore = "porting: project copy routes not implemented"]
async fn lists_directories_and_manages_git_worktree_copies() {
    let app = router(AppState::new());
    let project = send(&app, get("/project/current")).await;
    assert_eq!(project.status(), StatusCode::OK);
    let project_id = json(project).await["id"].as_str().expect("id").to_string();
    let base = format!("/project/{project_id}");
    let copies =
        format!("/experimental/project/{project_id}/copy?location%5Bdirectory%5D={DIRECTORY}");
    let created_parent = format!("{DIRECTORY}-http-copy");
    let created_directory = format!("{created_parent}/copy");

    let initial = send(&app, get(&format!("{base}/directories"))).await;
    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(
        json(initial).await,
        serde_json::json!([{ "directory": DIRECTORY }])
    );

    let generated = send(
        &app,
        post_json(
            &format!("/experimental/project/{project_id}/copy/generate-name"),
            &serde_json::json!({ "context": null }),
        ),
    )
    .await;
    assert_eq!(generated.status(), StatusCode::OK);
    assert!(json(generated).await["name"].is_string());

    let create = send(
        &app,
        post_json(
            &copies,
            &serde_json::json!({
                "strategy": "git_worktree",
                "directory": created_parent,
                "name": "copy",
            }),
        ),
    )
    .await;
    assert_eq!(create.status(), StatusCode::OK);
    assert_eq!(json(create).await["directory"], created_directory);

    let listed = send(&app, get(&format!("{base}/directories"))).await;
    assert_eq!(listed.status(), StatusCode::OK);
    assert!(json(listed)
        .await
        .as_array()
        .expect("array")
        .iter()
        .any(|item| item["directory"] == created_directory && item["strategy"] == "git_worktree"));

    let remove = send(
        &app,
        delete_json(
            &copies,
            &serde_json::json!({ "directory": created_directory, "force": false }),
        ),
    )
    .await;
    assert_eq!(remove.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(remove).await["data"]["forceRequired"], true);

    let forced = send(
        &app,
        delete_json(
            &copies,
            &serde_json::json!({ "directory": created_directory, "force": true }),
        ),
    )
    .await;
    assert_eq!(forced.status(), StatusCode::NO_CONTENT);
}
