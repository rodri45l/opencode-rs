//! Port of packages/opencode/test/server/httpapi-ui.test.ts (upstream 18ef3cc).
//!
//! Subset: the web UI fallback auth contract — the root serves HTML, a
//! configured password challenges with `401`/`WWW-Authenticate`, and CORS
//! preflight is allowed without auth. Cases that proxy upstream assets or read
//! embedded UI files need the UI bundle and are left to the UI phase.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{request, send};
use opencode_server::{router, router_with_options, AppState, AuthConfig, ServerOptions};

fn get(uri: &str) -> Request<Body> {
    request("GET", uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn serves_the_web_ui_through_the_http_api_app() {
    let app = router(AppState::new());
    let res = send(&app, get("/")).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers()["content-type"]
        .to_str()
        .unwrap_or_default()
        .contains("text/html"));
}

#[tokio::test]
async fn requires_server_password_for_the_web_ui() {
    let app = router_with_options(
        AppState::new(),
        ServerOptions::with_auth(AuthConfig::with_credentials("opencode", "secret")),
    );
    let res = send(&app, get("/")).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()["www-authenticate"],
        "Basic realm=\"Secure Area\""
    );
}

#[tokio::test]
async fn allows_web_ui_preflight_without_auth() {
    let app = router_with_options(
        AppState::new(),
        ServerOptions::with_auth(AuthConfig::with_credentials("opencode", "secret")),
    );
    let req = common::header(
        common::header(
            request("OPTIONS", "/").body(Body::empty()).unwrap(),
            "origin",
            "http://localhost:3000",
        ),
        "access-control-request-method",
        "GET",
    );
    let res = send(&app, req).await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "http://localhost:3000"
    );
}
