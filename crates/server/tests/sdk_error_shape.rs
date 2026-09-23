//! Port of packages/opencode/test/server/sdk-error-shape.test.ts (upstream 18ef3cc).
//!
//! Re-derived: the reference drives the generated SDK with `throwOnError`; the
//! Rust port asserts the underlying wire shape the SDK extracts its message
//! from — a NamedError body with a non-empty `data.message`.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-sdk-error-shape-port";

fn with_directory(req: Request<Body>) -> Request<Body> {
    common::header(req, "x-opencode-directory", DIRECTORY)
}

#[tokio::test]
#[ignore = "porting: session routes not implemented"]
async fn not_found_with_named_error_body_carries_the_server_message() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(
            request("GET", "/session/ses_no_such")
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let body = json(res).await;
    assert_eq!(body["name"], "NotFoundError");
    assert!(body["data"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("Session not found"));
}

#[tokio::test]
#[ignore = "porting: sync routes not implemented"]
async fn schema_rejection_extracts_the_field_level_reason() {
    let app = router(AppState::new());
    let res = send(
        &app,
        with_directory(
            request("POST", "/sync/history")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"aggregate":-1}"#))
                .unwrap(),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = json(res).await;
    assert_eq!(body["name"], "BadRequest");
    assert!(matches!(
        body["data"]["kind"].as_str(),
        Some("Body") | Some("Payload")
    ));
    assert!(body["data"]["message"]
        .as_str()
        .is_some_and(|message| !message.is_empty()));
}
