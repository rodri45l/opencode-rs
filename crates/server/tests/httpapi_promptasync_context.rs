//! Port of packages/opencode/test/server/httpapi-promptasync-context.test.ts (upstream 18ef3cc).
//!
//! Subset: the streaming prompt handler must preserve the request's instance
//! context (directory and selected workspace) while the response body runs
//! detached from the request fiber. The reference probes this with Effect
//! `forkIn`/`Stream.fromEffect` routes; the Rust port drives the real prompt
//! route and asserts the streamed response, so the cases stay red until the
//! prompt route and its context plumbing land.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-promptasync-context-port";

fn prompt_async(uri: &str, body: &serde_json::Value) -> Request<Body> {
    let req = json_body(request("POST", uri).body(Body::empty()).unwrap(), body);
    common::header(req, "x-opencode-directory", DIRECTORY)
}

#[tokio::test]
#[ignore = "porting: session prompt routes not implemented"]
async fn fork_preserves_instance_context_across_the_fork() {
    let app = router(AppState::new());
    let res = send(
        &app,
        prompt_async(
            "/session/ses_promptasync/fork?directory=%2Ftmp%2Fopencode-promptasync-context-port&workspace=wrk_prompt",
            &serde_json::json!({ "prompt": { "text": "hello" } }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "porting: session prompt routes not implemented"]
async fn streamed_prompt_body_keeps_the_explicitly_provided_context() {
    let app = router(AppState::new());
    let res = send(
        &app,
        prompt_async(
            "/session/ses_promptasync/stream?directory=%2Ftmp%2Fopencode-promptasync-context-port&workspace=wrk_prompt",
            &serde_json::json!({ "prompt": { "text": "hello" } }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap_or_default()
        .contains("text/event-stream"));
}
