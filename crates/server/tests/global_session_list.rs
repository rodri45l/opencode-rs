//! Port of packages/opencode/test/server/global-session-list.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference drives `Session.listGlobal` through an Effect
//! store; the Rust port asserts the same behaviour through the experimental
//! global session list route.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-global-session-list-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

async fn create_session(app: &axum::Router, title: &str) -> String {
    let res = send(
        app,
        with_directory(json_body(
            request("POST", "/session").body(Body::empty()).unwrap(),
            &serde_json::json!({ "title": title }),
        )),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    json(res).await["id"].as_str().expect("id").to_string()
}

#[tokio::test]
async fn lists_sessions_across_projects_with_project_metadata() {
    let app = router(AppState::new());
    let first = create_session(&app, "first-session").await;
    let second = create_session(&app, "second-session").await;

    let res = send(&app, get("/experimental/session?limit=200")).await;
    assert_eq!(res.status(), StatusCode::OK);
    let sessions = json(res).await;
    let ids: Vec<String> = sessions
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();

    assert!(ids.contains(&first));
    assert!(ids.contains(&second));
    for item in sessions.as_array().expect("array") {
        assert!(item["project"]["id"].is_string());
        assert!(item["project"]["worktree"].is_string());
    }
}

#[tokio::test]
async fn excludes_archived_sessions_by_default() {
    let app = router(AppState::new());
    let archived = create_session(&app, "archived-session").await;

    let archive = send(
        &app,
        with_directory(json_body(
            request("PATCH", &format!("/session/{archived}"))
                .body(Body::empty())
                .unwrap(),
            &serde_json::json!({ "time": { "archived": 1 } }),
        )),
    )
    .await;
    assert_eq!(archive.status(), StatusCode::OK);

    let res = send(&app, get("/experimental/session?limit=200")).await;
    assert_eq!(res.status(), StatusCode::OK);
    let ids: Vec<String> = json(res)
        .await
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();
    assert!(!ids.contains(&archived));

    let all = send(&app, get("/experimental/session?limit=200&archived=true")).await;
    assert_eq!(all.status(), StatusCode::OK);
    let all_ids: Vec<String> = json(all)
        .await
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();
    assert!(all_ids.contains(&archived));
}

#[tokio::test]
async fn supports_cursor_pagination() {
    let app = router(AppState::new());
    let first = create_session(&app, "page-one").await;
    let second = create_session(&app, "page-two").await;

    let page = send(
        &app,
        get(&format!(
            "/experimental/session?directory={DIRECTORY}&limit=1"
        )),
    )
    .await;
    assert_eq!(page.status(), StatusCode::OK);
    let page = json(page).await;
    assert_eq!(page.as_array().expect("array").len(), 1);
    assert_eq!(page[0]["id"], second);

    let cursor = page[0]["time"]["updated"]
        .as_i64()
        .or_else(|| {
            page[0]["time"]["updated"]
                .as_u64()
                .map(|value| value as i64)
        })
        .expect("updated time");
    let next = send(
        &app,
        get(&format!(
            "/experimental/session?directory={DIRECTORY}&limit=10&cursor={cursor}"
        )),
    )
    .await;
    assert_eq!(next.status(), StatusCode::OK);
    let ids: Vec<String> = json(next)
        .await
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();
    assert!(ids.contains(&first));
    assert!(!ids.contains(&second));
}
