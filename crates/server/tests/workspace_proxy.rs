//! Port of packages/opencode/test/server/workspace-proxy.test.ts (upstream 18ef3cc).
//!
//! Subset: the proxy's observable request rewriting — stripping
//! opencode-internal headers, merging extra headers, and negotiating websocket
//! URLs/sub-protocols — is ported green against `opencode_server::proxy_util`.
//! The streamed HTTP response and websocket forwarding cases need the proxy
//! middleware and stay red until it lands.

mod common;

use axum::http::{HeaderMap, HeaderValue};
use opencode_server::proxy_util;

fn headers(entries: &[(&str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in entries {
        map.insert(
            axum::http::HeaderName::from_bytes(name.as_bytes()).expect("header name"),
            HeaderValue::from_str(value).expect("header value"),
        );
    }
    map
}

#[test]
fn strips_opencode_internal_headers_and_merges_extra_headers() {
    let map = headers(&[
        ("x-opencode-directory", "/secret/path"),
        ("x-opencode-workspace", "ws_123"),
        ("x-custom", "preserved"),
    ]);
    let forwarded = proxy_util::headers(&map, &[("x-injected", "extra")]);

    assert!(forwarded.get("x-opencode-directory").is_none());
    assert!(forwarded.get("x-opencode-workspace").is_none());
    assert_eq!(forwarded.get("x-custom").unwrap(), "preserved");
    assert_eq!(forwarded.get("x-injected").unwrap(), "extra");
}

#[test]
fn websocket_target_is_derived_from_the_proxy_url() {
    assert_eq!(
        proxy_util::websocket_target_url("http://127.0.0.1:3000/base/echo"),
        "ws://127.0.0.1:3000/base/echo"
    );
}

#[test]
fn websocket_protocols_are_negotiated_from_the_request() {
    let map = headers(&[("sec-websocket-protocol", "chat")]);
    assert_eq!(proxy_util::websocket_protocols(&map), vec!["chat"]);
}

#[tokio::test]
#[ignore = "porting: workspace proxy middleware not implemented"]
async fn proxies_http_request_and_returns_streamed_response_with_status_and_headers() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use opencode_server::{router, AppState};

    let app = router(AppState::new());
    let res = common::send(
        &app,
        Request::builder()
            .method("POST")
            .uri("/session/abc?workspace=wrk_remote")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    assert_eq!(res.headers()["x-remote"], "yes");
    assert!(res.headers().get("content-encoding").is_none());
    assert!(res.headers().get("content-length").is_none());
}

#[tokio::test]
#[ignore = "porting: workspace proxy middleware not implemented"]
async fn returns_500_when_remote_is_unreachable() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use opencode_server::{router, AppState};

    let app = router(AppState::new());
    let res = common::send(
        &app,
        Request::builder()
            .method("GET")
            .uri("/session/abc?workspace=wrk_unreachable")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn proxies_bodyless_web_mutation_requests_as_an_empty_body() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use opencode_server::{router, AppState};

    let app = router(AppState::new());
    let res = common::send(
        &app,
        Request::builder()
            .method("POST")
            .uri("/session/abc/abort?workspace=wrk_remote")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}
