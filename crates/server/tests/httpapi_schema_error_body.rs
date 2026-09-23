//! Port of packages/opencode/test/server/httpapi-schema-error-body.test.ts (upstream 18ef3cc).
//!
//! Subset: payload and query schema rejections must return NamedError-shaped
//! JSON (never an empty body), and rejected payloads must be length-capped so
//! they cannot echo unbounded input. The response-encode case seeds a corrupt
//! stored row directly in the database and is left to the session phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send, text};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-schema-error-body-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

fn get(uri: &str) -> Request<Body> {
    with_directory(request("GET", uri).body(Body::empty()).unwrap())
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
#[ignore = "porting: sync routes not implemented"]
async fn payload_schema_rejection_returns_named_error_shaped_json() {
    let app = router(AppState::new());
    let res = send(&app, post_raw("/sync/history", r#"{"aggregate":-1}"#)).await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap_or_default()
        .contains("application/json"));
    let parsed = json(res).await;
    assert_eq!(parsed["name"], "BadRequest");
    assert!(matches!(
        parsed["data"]["kind"].as_str(),
        Some("Body") | Some("Payload")
    ));
    assert!(parsed["data"]["message"]
        .as_str()
        .is_some_and(|m| !m.is_empty()));
}

#[tokio::test]
#[ignore = "porting: filesystem routes not implemented"]
async fn query_schema_rejection_returns_named_error_shaped_json() {
    let app = router(AppState::new());
    let res = send(
        &app,
        get(&format!(
            "/find/file?query=foo&limit=999999&directory={DIRECTORY}"
        )),
    )
    .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let parsed = json(res).await;
    assert_eq!(parsed["name"], "BadRequest");
    assert_eq!(parsed["data"]["kind"], "Query");
}

#[tokio::test]
#[ignore = "porting: v2 session routes not implemented"]
async fn v2_query_schema_rejection_returns_invalid_request_error_json() {
    let app = router(AppState::new());
    let res = send(&app, get("/api/session?limit=0")).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let parsed = json(res).await;

    assert_eq!(parsed["_tag"], "InvalidRequestError");
    assert_eq!(parsed["kind"], "Query");
    assert!(parsed["message"].as_str().is_some_and(|m| !m.is_empty()));
}

#[tokio::test]
#[ignore = "porting: sync routes not implemented"]
async fn rejected_request_body_is_capped_and_does_not_echo_unbounded_input() {
    let app = router(AppState::new());
    let huge = "X".repeat(50_000);
    let res = send(
        &app,
        post_raw("/sync/history", &format!(r#"{{"aggregate":"{huge}"}}"#)),
    )
    .await;
    let status = res.status();
    let body = text(res).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body.len() < 2 * 1024,
        "error body must be capped, was {}",
        body.len()
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("json body");
    assert!(!parsed["data"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains(&huge));
}
