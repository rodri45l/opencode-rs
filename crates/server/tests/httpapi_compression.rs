//! Port of packages/opencode/test/server/httpapi-compression.test.ts (upstream 18ef3cc).
//!
//! Subset: compression negotiation on large JSON responses and the SSE
//! exclusions. The reference decompresses bodies with node:zlib; the Rust port
//! asserts the `content-encoding` decision (the crate has no compression
//! dependency). The `/event` exclusion is already green because the SSE route
//! exists and is never compressed.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{request, send};
use opencode_server::{router, AppState};

const DIRECTORY: &str = "/tmp/opencode-httpapi-compression-port";

fn fat_config() -> serde_json::Value {
    let instructions: Vec<String> = (0..50)
        .map(|i| format!("padding-instruction-{i}-{}", "x".repeat(40)))
        .collect();
    serde_json::json!({
        "formatter": false,
        "lsp": false,
        "username": "compression-test-user",
        "instructions": instructions,
    })
}

fn get_with_encoding(uri: &str, encoding: Option<&str>) -> Request<Body> {
    let mut builder = request("GET", uri);
    builder = builder.header("x-opencode-directory", DIRECTORY);
    if let Some(encoding) = encoding {
        builder = builder.header("accept-encoding", encoding);
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn gzips_json_when_accept_encoding_includes_gzip_and_body_exceeds_threshold() {
    let _ = fat_config();
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/config", Some("gzip"))).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-encoding"], "gzip");
}

#[tokio::test]
async fn uses_deflate_when_only_deflate_is_acceptable() {
    let _ = fat_config();
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/config", Some("deflate"))).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-encoding"], "deflate");
}

#[tokio::test]
async fn prefers_gzip_when_both_gzip_and_deflate_are_acceptable() {
    let _ = fat_config();
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/config", Some("gzip, deflate"))).await;

    assert_eq!(res.headers()["content-encoding"], "gzip");
}

#[tokio::test]
async fn skips_when_no_accept_encoding_header_is_present() {
    let _ = fat_config();
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/config", None)).await;

    assert!(res.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn skips_when_accept_encoding_only_allows_unsupported_encodings() {
    let _ = fat_config();
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/config", Some("br"))).await;

    assert!(res.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn skips_head_requests() {
    let _ = fat_config();
    let app = router(AppState::new());
    let req = common::header(
        request("HEAD", "/config").body(Body::empty()).unwrap(),
        "accept-encoding",
        "gzip",
    );
    let res = send(&app, req).await;

    assert!(res.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn event_sse_is_not_compressed() {
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/event", Some("gzip"))).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn global_event_sse_is_not_compressed() {
    let app = router(AppState::new());
    let res = send(&app, get_with_encoding("/global/event", Some("gzip"))).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("content-encoding").is_none());
}
