//! Port of packages/opencode/test/server/httpapi-cors-vary.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the CORS preflight `Vary` handling; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{header, request, send};
use opencode_server::{router, AppState};

fn preflight() -> Request<Body> {
    let req = request("OPTIONS", "/global/config")
        .body(Body::empty())
        .unwrap();
    let req = header(req, "origin", "http://localhost:3000");
    let req = header(req, "access-control-request-method", "POST");
    header(
        req,
        "access-control-request-headers",
        "content-type, x-opencode-directory",
    )
}

fn vary_count(vary: &str, needle: &str) -> usize {
    vary.split(',')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| part == needle)
        .count()
}

#[tokio::test]
async fn preflight_vary_contains_origin() {
    let app = router(AppState::new());
    let res = send(&app, preflight()).await;

    assert!(
        res.status() == StatusCode::OK || res.status() == StatusCode::NO_CONTENT,
        "unexpected status: {}",
        res.status()
    );
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "http://localhost:3000"
    );
    assert!(res.headers()["vary"]
        .to_str()
        .unwrap_or_default()
        .to_lowercase()
        .contains("origin"));
}

#[tokio::test]
async fn preflight_vary_preserves_access_control_request_headers() {
    let app = router(AppState::new());
    let res = send(&app, preflight()).await;

    let vary = res.headers()["vary"]
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    assert!(vary.contains("origin"), "vary was: {vary}");
    assert!(
        vary.contains("access-control-request-headers"),
        "vary was: {vary}"
    );
}

#[tokio::test]
async fn preflight_vary_does_not_duplicate_origin() {
    let app = router(AppState::new());
    let res = send(&app, preflight()).await;

    let vary = res.headers()["vary"].to_str().unwrap_or_default();
    assert_eq!(vary_count(vary, "origin"), 1, "vary was: {vary}");
}
