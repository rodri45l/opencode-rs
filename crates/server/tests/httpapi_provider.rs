//! Port of packages/opencode/test/server/httpapi-provider.test.ts (upstream 18ef3cc).
//!
//! Subset: the declared provider OAuth callback errors and the provider/config
//! catalog list shape. Cases that install project plugins or mutate provider
//! model hooks require the plugin loader and are left to the plugin phase. The
//! reference's `it.instance.skip` v2 not-found case is intentionally not
//! ported here.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-provider-port";
const PROVIDER_ID: &str = "test-oauth-parity";

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
#[ignore = "porting: provider oauth routes not implemented"]
async fn returns_declared_provider_auth_callback_errors() {
    let app = router(AppState::new());
    let res = send(
        &app,
        post_json(
            &format!("/provider/{PROVIDER_ID}/oauth/callback"),
            &serde_json::json!({ "method": 0 }),
        ),
    )
    .await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json(res).await,
        serde_json::json!({
            "name": "ProviderAuthOauthMissing",
            "data": { "providerID": PROVIDER_ID },
        })
    );
}

#[tokio::test]
#[ignore = "porting: provider routes not implemented"]
async fn serves_provider_and_config_provider_lists() {
    let app = router(AppState::new());

    let provider = send(&app, get("/provider")).await;
    assert_eq!(provider.status(), StatusCode::OK);
    assert!(json(provider).await["all"].is_array());

    let config = send(&app, get("/config/providers")).await;
    assert_eq!(config.status(), StatusCode::OK);
    assert!(json(config).await["providers"].is_array());
}
