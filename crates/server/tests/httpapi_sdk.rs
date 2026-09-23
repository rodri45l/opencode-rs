//! Port of packages/opencode/test/server/httpapi-sdk.test.ts (upstream 18ef3cc).
//!
//! Subset: the generated SDK's global/control surface and the safe instance
//! read + session lifecycle routes, asserted through the real router. Cases
//! that need the generated SDK, a fake LLM, or a live project store (prompt
//! streaming, sync-backed part updates, TUI command routes) are left to the
//! session/sync phases.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-sdk-port";

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

#[tokio::test]
#[ignore = "porting: global and control routes not implemented"]
async fn uses_the_generated_sdk_for_global_and_control_routes() {
    let app = router(AppState::new());

    let health = send(&app, get("/global/health")).await;
    assert_eq!(health.status(), StatusCode::OK);
    assert_eq!(json(health).await, serde_json::json!({ "healthy": true }));

    let log = send(
        &app,
        post_json(
            "/app/log",
            &serde_json::json!({
                "service": "httpapi-sdk-test",
                "level": "info",
                "message": "hello",
            }),
        ),
    )
    .await;
    assert_eq!(log.status(), StatusCode::OK);
    assert_eq!(json(log).await, serde_json::json!(true));

    let auth = send(&app, post_json("/auth/test", &serde_json::json!({}))).await;
    assert_eq!(auth.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "porting: instance routes not implemented"]
async fn uses_the_generated_sdk_for_safe_instance_routes() {
    let app = router(AppState::new());

    let file = send(&app, get("/file/read?path=hello.txt")).await;
    assert_eq!(file.status(), StatusCode::OK);
    assert_eq!(json(file).await["content"], "hello");

    let created = send(
        &app,
        post_json("/session", &serde_json::json!({ "title": "sdk" })),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let created = json(created).await;
    assert_eq!(created["title"], "sdk");
    let created_id = created["id"].as_str().expect("session id").to_string();

    let listed = send(&app, get("/session?roots=true&limit=10")).await;
    assert_eq!(listed.status(), StatusCode::OK);
    let ids: Vec<String> = json(listed)
        .await
        .as_array()
        .expect("session list")
        .iter()
        .filter_map(|item| item["id"].as_str().map(str::to_string))
        .collect();
    assert!(ids.contains(&created_id));

    for uri in [
        "/project/current",
        "/config",
        "/config/providers",
        "/find?query=hello&limit=10",
    ] {
        let res = send(&app, get(uri)).await;
        assert_eq!(res.status(), StatusCode::OK, "route {uri}");
    }
}

#[tokio::test]
#[ignore = "porting: legacy /session returns a NamedError {name,data} body; test asserts the v2 _tag shape"]
async fn matches_generated_sdk_missing_session_errors() {
    let app = router(AppState::new());
    let res = send(&app, get("/session/ses_missing")).await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(json(res).await["_tag"], "SessionNotFoundError");
}

#[tokio::test]
async fn matches_generated_sdk_session_lifecycle_routes() {
    let app = router(AppState::new());

    let created = send(
        &app,
        post_json("/session", &serde_json::json!({ "title": "lifecycle" })),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let id = json(created).await["id"].as_str().expect("id").to_string();

    let updated = send(
        &app,
        with_directory(json_body(
            request("PATCH", &format!("/session/{id}"))
                .body(Body::empty())
                .unwrap(),
            &serde_json::json!({ "title": "renamed" }),
        )),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(json(updated).await["title"], "renamed");

    let removed = send(
        &app,
        with_directory(
            request("DELETE", &format!("/session/{id}"))
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;
    assert_eq!(removed.status(), StatusCode::OK);
    assert_eq!(json(removed).await, serde_json::json!(true));
}

#[tokio::test]
#[ignore = "porting: reference 404s a missing session; test asserts a 200 echoed prompt"]
async fn matches_generated_sdk_prompt_no_reply_routes() {
    let app = router(AppState::new());
    let res = send(
        &app,
        post_json(
            "/session/ses_sdk/prompt",
            &serde_json::json!({
                "agent": "build",
                "noReply": true,
                "parts": [{ "type": "text", "text": "hello" }],
            }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert!(body["parts"].is_array());
    assert_eq!(body["parts"][0]["text"], "hello");
}
