//! Port of packages/opencode/test/server/httpapi-cors.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the CORS middleware; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{header, request, send};
use opencode_server::{router, router_with_options, AppState, AuthConfig, ServerOptions};

fn preflight(origin: &str) -> Request<Body> {
    let req = request("OPTIONS", "/").body(Body::empty()).unwrap();
    let req = header(req, "origin", origin);
    let req = header(req, "access-control-request-method", "GET");
    header(req, "access-control-request-headers", "authorization")
}

#[tokio::test]
#[ignore = "porting: httpapi cors middleware not implemented"]
async fn allows_browser_preflight_requests_without_credentials() {
    let app = router(AppState::new());
    let res = send(&app, preflight("http://localhost:3000")).await;

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "http://localhost:3000"
    );
    assert_eq!(
        res.headers()["access-control-allow-headers"],
        "authorization"
    );
}

#[tokio::test]
#[ignore = "porting: httpapi cors middleware not implemented"]
async fn adds_cors_headers_to_unauthorized_responses() {
    let app = router_with_options(
        AppState::new(),
        ServerOptions::with_auth(AuthConfig::with_password("secret")),
    );
    let req = header(
        request("GET", "/global/config")
            .body(Body::empty())
            .unwrap(),
        "origin",
        "https://app.opencode.ai",
    );
    let res = send(&app, req).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "https://app.opencode.ai"
    );
}

#[tokio::test]
#[ignore = "porting: httpapi cors middleware not implemented"]
async fn uses_custom_cors_origins_passed_to_the_server() {
    let app = router_with_options(
        AppState::new(),
        ServerOptions::with_cors(vec!["https://custom.example".to_string()]),
    );

    let allowed = send(&app, preflight("https://custom.example")).await;
    assert_eq!(allowed.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        allowed.headers()["access-control-allow-origin"],
        "https://custom.example"
    );
    assert_eq!(
        allowed.headers()["access-control-allow-headers"],
        "authorization"
    );

    let rejected = send(&app, preflight("https://evil.example")).await;
    assert_eq!(rejected.status(), StatusCode::NO_CONTENT);
    assert_ne!(
        rejected.headers()["access-control-allow-origin"],
        "https://evil.example"
    );
}
