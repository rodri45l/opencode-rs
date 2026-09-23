//! Port of packages/opencode/test/server/httpapi-config.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the config GET/PATCH routes; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

fn directory() -> String {
    let dir = std::env::temp_dir().join("opencode-config-port");
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.to_string_lossy().into_owned()
}

fn with_directory(req: Request<Body>, dir: &str) -> Request<Body> {
    common::header(req, "x-opencode-directory", dir)
}

#[tokio::test]
async fn serves_config_update_through_the_default_server_app() {
    let dir = directory();
    let app = router(AppState::new());
    let body = serde_json::json!({
        "username": "patched-user",
        "formatter": false,
        "lsp": false,
    });
    let req = with_directory(
        json_body(
            request("PATCH", "/config").body(Body::empty()).unwrap(),
            &body,
        ),
        &dir,
    );

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    let patched = json(res).await;
    assert_eq!(patched["username"], "patched-user");
    assert_eq!(patched["formatter"], false);
    assert_eq!(patched["lsp"], false);

    let saved = std::fs::read_to_string(std::path::Path::new(&dir).join("config.json"))
        .expect("config.json written");
    let saved: serde_json::Value = serde_json::from_str(&saved).expect("valid json");
    assert_eq!(saved["username"], "patched-user");
    assert_eq!(saved["formatter"], false);
    assert_eq!(saved["lsp"], false);
}

#[tokio::test]
async fn serves_config_with_active_provider_model_status() {
    let dir = directory();
    let app = router(AppState::new());
    let req = with_directory(request("GET", "/config").body(Body::empty()).unwrap(), &dir);

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json(res).await;
    assert_eq!(
        body["provider"]["omniroute"]["models"]["gpt-4o"]["status"],
        "active"
    );
}
