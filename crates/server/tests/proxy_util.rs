//! Port of packages/opencode/test/server/proxy-util.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `ProxyUtil`; see docs/TEST-PORT.md.
//!
//! Re-derived against `opencode_server::proxy_util`: the reference accepts
//! `URL`/`Request` objects, the Rust surface accepts the equivalent
//! `&str`/`HeaderMap` inputs.

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
fn websocket_target_url_converts_http_to_ws() {
    assert_eq!(
        proxy_util::websocket_target_url("http://example.com/path"),
        "ws://example.com/path"
    );
}

#[test]
fn websocket_target_url_converts_https_to_wss() {
    assert_eq!(
        proxy_util::websocket_target_url("https://example.com/path"),
        "wss://example.com/path"
    );
}

#[test]
fn websocket_target_url_preserves_query_params() {
    assert_eq!(
        proxy_util::websocket_target_url("http://example.com/path?foo=bar"),
        "ws://example.com/path?foo=bar"
    );
}

#[test]
fn websocket_target_url_accepts_bare_targets() {
    assert_eq!(
        proxy_util::websocket_target_url("ws://localhost:3000/ws"),
        "ws://localhost:3000/ws"
    );
}

#[test]
fn websocket_protocols_returns_empty_when_absent() {
    assert_eq!(
        proxy_util::websocket_protocols(&HeaderMap::new()),
        Vec::<String>::new()
    );
}

#[test]
fn websocket_protocols_parses_single_protocol() {
    let map = headers(&[("sec-websocket-protocol", "graphql-ws")]);
    assert_eq!(proxy_util::websocket_protocols(&map), vec!["graphql-ws"]);
}

#[test]
fn websocket_protocols_parses_multiple_protocols() {
    let map = headers(&[("sec-websocket-protocol", "graphql-ws, graphql-transport-ws")]);
    assert_eq!(
        proxy_util::websocket_protocols(&map),
        vec!["graphql-ws", "graphql-transport-ws"]
    );
}

#[test]
fn websocket_protocols_trims_whitespace_and_filters_empty() {
    let map = headers(&[("sec-websocket-protocol", " proto1 , , proto2 ")]);
    assert_eq!(
        proxy_util::websocket_protocols(&map),
        vec!["proto1", "proto2"]
    );
}

#[test]
fn headers_strips_hop_by_hop_headers() {
    let map = headers(&[
        ("connection", "keep-alive"),
        ("keep-alive", "timeout=5"),
        ("transfer-encoding", "chunked"),
        ("content-type", "application/json"),
    ]);
    let result = proxy_util::headers(&map, &[]);
    assert!(result.get("connection").is_none());
    assert!(result.get("keep-alive").is_none());
    assert!(result.get("transfer-encoding").is_none());
    assert_eq!(result.get("content-type").unwrap(), "application/json");
}

#[test]
fn headers_strips_opencode_specific_headers() {
    let map = headers(&[
        ("x-opencode-directory", "/home/user/project"),
        ("x-opencode-workspace", "ws_123"),
        ("accept-encoding", "gzip"),
        ("x-custom", "keep"),
    ]);
    let result = proxy_util::headers(&map, &[]);
    assert!(result.get("x-opencode-directory").is_none());
    assert!(result.get("x-opencode-workspace").is_none());
    assert!(result.get("accept-encoding").is_none());
    assert_eq!(result.get("x-custom").unwrap(), "keep");
}

#[test]
fn headers_merges_extra_headers() {
    let map = headers(&[("content-type", "application/json")]);
    let result = proxy_util::headers(&map, &[("x-auth", "token"), ("content-type", "text/plain")]);
    assert_eq!(result.get("x-auth").unwrap(), "token");
    assert_eq!(result.get("content-type").unwrap(), "text/plain");
}

#[test]
fn headers_returns_original_headers_when_no_extra() {
    let map = headers(&[("content-type", "application/json"), ("x-foo", "bar")]);
    let result = proxy_util::headers(&map, &[]);
    assert_eq!(result.get("content-type").unwrap(), "application/json");
    assert_eq!(result.get("x-foo").unwrap(), "bar");
}
