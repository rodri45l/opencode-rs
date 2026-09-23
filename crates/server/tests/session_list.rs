//! Port of packages/opencode/test/server/session-list.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference drives `Session.list` through an Effect instance
//! store; the Rust port asserts the same filtering behaviour through the
//! `/session` list route and its query parameters.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-session-list-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
}

async fn create_session(app: &axum::Router, body: serde_json::Value) -> String {
    let res = send(
        app,
        with_directory(json_body(
            request("POST", "/session").body(Body::empty()).unwrap(),
            &body,
        )),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    json(res).await["id"].as_str().expect("id").to_string()
}

#[tokio::test]
async fn filters_root_sessions() {
    let app = router(AppState::new());
    let root = create_session(&app, serde_json::json!({ "title": "root-session" })).await;
    let child = create_session(
        &app,
        serde_json::json!({ "title": "child-session", "parentID": root }),
    )
    .await;

    let res = send(&app, get("/session?roots=true")).await;
    assert_eq!(res.status(), StatusCode::OK);
    let ids: Vec<String> = json(res)
        .await
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();

    assert!(ids.contains(&root));
    assert!(!ids.contains(&child));
}

#[tokio::test]
async fn filters_by_start_time() {
    let app = router(AppState::new());
    let _ = create_session(&app, serde_json::json!({ "title": "new-session" })).await;

    let future = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        + 86_400_000;
    let res = send(&app, get(&format!("/session?start={future}"))).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(json(res).await.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn filters_by_search_term() {
    let app = router(AppState::new());
    let _ = create_session(
        &app,
        serde_json::json!({ "title": "unique-search-term-abc" }),
    )
    .await;
    let _ = create_session(&app, serde_json::json!({ "title": "other-session-xyz" })).await;

    let res = send(&app, get("/session?search=unique-search")).await;
    assert_eq!(res.status(), StatusCode::OK);
    let titles: Vec<String> = json(res)
        .await
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|item| item["title"].as_str().map(str::to_string))
        .collect();

    assert!(titles.iter().any(|title| title == "unique-search-term-abc"));
    assert!(!titles.iter().any(|title| title == "other-session-xyz"));
}

#[tokio::test]
async fn respects_limit_parameter() {
    let app = router(AppState::new());
    for index in 1..=3 {
        let _ = create_session(
            &app,
            serde_json::json!({ "title": format!("session-{index}") }),
        )
        .await;
    }

    let res = send(&app, get("/session?limit=2")).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await.as_array().expect("array").len(), 2);
}

#[tokio::test]
async fn includes_metadata_in_listed_sessions() {
    let app = router(AppState::new());
    let meta = serde_json::json!({ "source": "sdk", "trace": { "id": "abc" } });
    let created = create_session(
        &app,
        serde_json::json!({ "title": "meta-session", "metadata": meta }),
    )
    .await;

    let res = send(&app, get("/session?search=meta-session")).await;
    assert_eq!(res.status(), StatusCode::OK);
    let listed = json(res).await;
    let item = listed
        .as_array()
        .expect("array")
        .iter()
        .find(|item| item["id"] == created)
        .expect("created session");
    assert_eq!(item["metadata"], meta);
}
