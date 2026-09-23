//! Port of packages/opencode/test/server/httpapi-global.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the global upgrade route; see docs/TEST-PORT.md.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{json, json_body, request, send};
use opencode_server::{router, AppState};

fn upgrade(body: &serde_json::Value) -> Request<Body> {
    json_body(
        request("POST", "/global/upgrade")
            .body(Body::empty())
            .unwrap(),
        body,
    )
}

#[tokio::test]
#[ignore = "porting: global upgrade route not implemented"]
async fn upgrades_to_the_requested_version() {
    let app = router(AppState::new());
    let res = send(&app, upgrade(&serde_json::json!({ "target": "9.9.9" }))).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        json(res).await,
        serde_json::json!({ "success": true, "version": "9.9.9" })
    );
}

#[tokio::test]
#[ignore = "porting: global upgrade route not implemented"]
async fn rejects_invalid_upgrade_payloads() {
    let app = router(AppState::new());
    let res = send(&app, upgrade(&serde_json::json!({ "target": 1 }))).await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "porting: global upgrade route not implemented"]
async fn rejects_invalid_upgrade_target_versions() {
    let app = router(AppState::new());
    let res = send(&app, upgrade(&serde_json::json!({ "target": "latest" }))).await;

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "porting: global upgrade route not implemented"]
async fn rejects_unsupported_upgrade_content_types() {
    let app = router(AppState::new());
    let req = request("POST", "/global/upgrade")
        .body(Body::empty())
        .unwrap();
    let req = common::header(req, "content-type", "text/plain");
    let mut req = req;
    *req.body_mut() = Body::from("{\"target\":\"1.0.0\"}");

    let res = send(&app, req).await;
    assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
