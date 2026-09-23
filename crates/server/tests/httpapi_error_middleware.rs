//! Port of packages/opencode/test/server/httpapi-error-middleware.test.ts (upstream 18ef3cc).
//!
//! Subset: the safe-body contract for unknown 500 defects, the structured 400
//! bodies for invalid-config and remote-auth defects, and storage not-found
//! staying a 500. The reference injects throwing routes into a test router; the
//! Rust port drives the real router and asserts the same wire bodies, so the
//! cases stay red until the error middleware lands.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, request, send};
use opencode_server::{router, AppState};

fn get(uri: &str) -> Request<Body> {
    request("GET", uri).body(Body::empty()).unwrap()
}

fn assert_unknown_error_body(body: &serde_json::Value) {
    assert_eq!(body["name"], "UnknownError");
    assert_eq!(
        body["data"]["message"],
        "Unexpected server error. Check server logs for details."
    );
    let reference = body["data"]["ref"].as_str().expect("error ref");
    assert!(reference.starts_with("err_"), "ref prefix: {reference}");
    let suffix = &reference["err_".len()..];
    assert_eq!(suffix.len(), 8, "ref suffix length: {reference}");
    assert!(
        suffix.chars().all(|c| c.is_ascii_hexdigit() || c == '-'),
        "ref suffix alphabet: {reference}"
    );
}

#[tokio::test]
async fn returns_a_safe_body_for_unknown_500_defects() {
    let app = router(AppState::new());
    let res = send(&app, get("/boom")).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = json(res).await;
    assert_unknown_error_body(&body);
    assert!(!body.to_string().contains("secret stack marker"));
}

#[tokio::test]
async fn returns_a_safe_body_for_named_defects() {
    let app = router(AppState::new());
    let res = send(&app, get("/named")).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = json(res).await;
    assert_unknown_error_body(&body);
    assert!(!body.to_string().contains("secret named marker"));
}

#[tokio::test]
async fn returns_invalid_config_defects_as_structured_client_errors() {
    let app = router(AppState::new());
    let res = send(&app, get("/config-error")).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body = json(res).await;
    assert_eq!(body["name"], "ConfigInvalidError");
    assert_eq!(body["data"]["path"], "/tmp/opencode.json");
    assert_eq!(body["data"]["issues"][0]["message"], "Expected object");
    assert_eq!(
        body["data"]["issues"][0]["path"],
        serde_json::json!(["provider", "anthropic", "options"])
    );
    let serialized = body.to_string();
    assert!(serialized.contains("/tmp/opencode.json"));
    assert!(serialized.contains("anthropic"));
}

#[tokio::test]
async fn returns_remote_auth_defects_as_structured_client_errors() {
    let app = router(AppState::new());
    let res = send(&app, get("/remote-auth-error")).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "name": "RemoteAuthError",
            "data": {
                "url": "https://example.com",
                "remote": "https://config.example.com/opencode.json",
            },
        })
    );
}

#[tokio::test]
async fn does_not_map_storage_not_found_defects_to_404() {
    let app = router(AppState::new());
    let res = send(&app, get("/missing")).await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = json(res).await;
    assert_unknown_error_body(&body);
}
