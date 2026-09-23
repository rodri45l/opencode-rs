//! Port of packages/opencode/test/server/httpapi-experimental.test.ts (upstream 18ef3cc).
//!
//! Subset: read-only experimental endpoints and the declared worktree error.
//! The Console account switch, global session list, and worktree mutation
//! cases need the account database and a git worktree; those are left to the
//! control-plane phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-experimental-port";

fn get(uri: &str) -> Request<Body> {
    common::header(
        request("GET", uri).body(Body::empty()).unwrap(),
        "x-opencode-directory",
        DIRECTORY,
    )
}

#[tokio::test]
#[ignore = "porting: experimental read-only routes not implemented"]
async fn serves_read_only_experimental_endpoints() {
    let app = router(AppState::new());

    let console_state = send(&app, get("/experimental/console")).await;
    assert_eq!(console_state.status(), StatusCode::OK);
    assert_eq!(
        json(console_state).await,
        serde_json::json!({ "consoleManagedProviders": [], "switchableOrgCount": 0 })
    );

    let console_orgs = send(&app, get("/experimental/console/orgs")).await;
    assert_eq!(console_orgs.status(), StatusCode::OK);
    assert_eq!(json(console_orgs).await, serde_json::json!({ "orgs": [] }));

    let tool_list = send(
        &app,
        get("/experimental/tool?provider=opencode&model=gpt-5"),
    )
    .await;
    assert_eq!(tool_list.status(), StatusCode::OK);
    let tools = json(tool_list).await;
    assert!(tools.as_array().expect("tool array").iter().any(|tool| {
        tool["id"] == "bash" && tool["description"].is_string() && tool["parameters"].is_object()
    }));

    let tool_ids = send(&app, get("/experimental/tool/ids")).await;
    assert_eq!(tool_ids.status(), StatusCode::OK);
    assert!(json(tool_ids).await.to_string().contains("bash"));

    let worktrees = send(&app, get("/experimental/worktree")).await;
    assert_eq!(worktrees.status(), StatusCode::OK);
    assert_eq!(json(worktrees).await, serde_json::json!([]));

    let resources = send(&app, get("/experimental/resource")).await;
    assert_eq!(resources.status(), StatusCode::OK);
    assert_eq!(json(resources).await, serde_json::json!({}));
}

#[tokio::test]
#[ignore = "porting: experimental worktree route not implemented"]
async fn returns_declared_worktree_errors() {
    let app = router(AppState::new());
    let req = common::header(
        json_body(
            request("POST", "/experimental/worktree")
                .body(Body::empty())
                .unwrap(),
            &serde_json::json!({}),
        ),
        "x-opencode-directory",
        DIRECTORY,
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "name": "WorktreeNotGitError",
            "data": { "message": "Worktrees are only supported for git projects" },
        })
    );
}
