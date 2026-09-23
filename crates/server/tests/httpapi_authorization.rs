//! Port of packages/opencode/test/server/httpapi-authorization.test.ts (upstream 18ef3cc).
//!
//! The reference file mounts a synthetic `probe`/`missing` HttpApi and applies
//! the authorization middleware. Re-expressed against the Rust router's real
//! paths (`/config` for `probe`, `/session/ses_missing` for the handler-error
//! case, `/api/session` for the v2 body). The middleware contract is unchanged:
//! 401 + `www-authenticate: Basic`, username matching, `auth_token` query
//! credentials, and the v2 `UnauthorizedError` body.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{basic, header, json, request, send};
use opencode_server::{router_with_options, AppState, AuthConfig, ServerOptions};

fn auth(config: AuthConfig) -> axum::Router {
    router_with_options(AppState::new(), ServerOptions::with_auth(config))
}

fn get(uri: &str, headers: &[(&str, &str)]) -> Request<Body> {
    let mut req = request("GET", uri).body(Body::empty()).unwrap();
    for (name, value) in headers {
        req = header(req, name, value);
    }
    req
}

#[tokio::test]
async fn allows_requests_when_server_password_is_not_configured() {
    let app = auth(AuthConfig::none());
    let res = send(&app, get("/config", &[])).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn requires_configured_password_for_basic_auth() {
    let app = auth(AuthConfig::with_password("secret"));

    let missing = send(&app, get("/config", &[])).await;
    assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
    assert!(missing.headers()["www-authenticate"]
        .to_str()
        .unwrap_or_default()
        .contains("Basic"));

    let bad_password = send(
        &app,
        get("/config", &[("authorization", &basic("opencode", "wrong"))]),
    )
    .await;
    assert_eq!(bad_password.status(), StatusCode::UNAUTHORIZED);
    assert!(bad_password.headers()["www-authenticate"]
        .to_str()
        .unwrap_or_default()
        .contains("Basic"));

    let good = send(
        &app,
        get(
            "/config",
            &[("authorization", &basic("opencode", "secret"))],
        ),
    )
    .await;
    assert_eq!(good.status(), StatusCode::OK);
}

#[tokio::test]
async fn respects_configured_basic_auth_username() {
    let app = auth(AuthConfig::with_credentials("kit", "secret"));

    let default_user = send(
        &app,
        get(
            "/config",
            &[("authorization", &basic("opencode", "secret"))],
        ),
    )
    .await;
    assert_eq!(default_user.status(), StatusCode::UNAUTHORIZED);

    let configured_user = send(
        &app,
        get("/config", &[("authorization", &basic("kit", "secret"))]),
    )
    .await;
    assert_eq!(configured_user.status(), StatusCode::OK);
}

#[tokio::test]
async fn accepts_auth_token_query_credentials() {
    let app = auth(AuthConfig::with_password("secret"));
    let token = common::base64("opencode:secret");
    let res = send(&app, get(&format!("/config?auth_token={token}"), &[])).await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn prefers_auth_token_query_credentials_over_basic_auth() {
    let app = auth(AuthConfig::with_password("secret"));
    let token = common::base64("opencode:secret");
    let res = send(
        &app,
        get(
            &format!("/config?auth_token={token}"),
            &[("authorization", &basic("opencode", "wrong"))],
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn preserves_handler_errors_when_basic_auth_succeeds() {
    let app = auth(AuthConfig::with_password("secret"));
    let res = send(
        &app,
        get(
            "/session/ses_missing",
            &[("authorization", &basic("opencode", "secret"))],
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    // The auth middleware must not swallow the handler's own not-found body.
    let body = json(res).await;
    assert_eq!(body["name"], "NotFoundError");
    assert!(body["data"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("Session not found"));
}

#[tokio::test]
async fn preserves_handler_errors_when_auth_token_query_succeeds() {
    let app = auth(AuthConfig::with_password("secret"));
    let token = common::base64("opencode:secret");
    let res = send(
        &app,
        get(&format!("/session/ses_missing?auth_token={token}"), &[]),
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
async fn rejects_malformed_auth_token_query_credentials() {
    let app = auth(AuthConfig::with_password("secret"));
    let res = send(&app, get("/config?auth_token=not-base64", &[])).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_bodyful_v2_unauthorized_errors() {
    let app = auth(AuthConfig::with_password("secret"));
    let res = send(&app, get("/api/session", &[])).await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert!(res.headers()["www-authenticate"]
        .to_str()
        .unwrap_or_default()
        .contains("Basic"));
    let body = json(res).await;
    assert_eq!(
        body,
        serde_json::json!({ "_tag": "UnauthorizedError", "message": "Authentication required" })
    );
}
