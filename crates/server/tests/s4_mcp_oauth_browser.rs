//! Port of packages/opencode/test/mcp/oauth-browser.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/mcp/index.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure `BrowserOpenFailed` event contract — its type name and
//! payload shape, and that a discovered authorization URL carries the registered
//! `client_id`.
//! Dropped: the live OAuth browser flow (publishing the failure event when
//! `xdg-open` fails, not publishing it on success, browser receiving the
//! discovered authorization URL). Those need an HTTP MCP server, a browser
//! service, and the callback listener.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn browser_open_failed_event_type() -> Result<String, NotImplemented> {
    nope("mcp oauth-browser")
}

fn browser_open_failed_payload(_mcp_name: &str, _url: &str) -> Result<Value, NotImplemented> {
    nope("mcp oauth-browser")
}

fn authorization_url_has_client_id(_url: &str, _client_id: &str) -> Result<bool, NotImplemented> {
    nope("mcp oauth-browser")
}

#[test]
#[ignore = "porting: mcp oauth-browser not implemented"]
fn browser_open_failed_event_is_published_when_browser_launch_fails() {
    assert_eq!(
        browser_open_failed_event_type().unwrap(),
        "BrowserOpenFailed"
    );
    assert_eq!(
        browser_open_failed_payload(
            "test-oauth-server",
            "http://127.0.0.1:1234/authorize?client_id=test-client"
        )
        .unwrap(),
        json!({ "mcpName": "test-oauth-server", "url": "http://127.0.0.1:1234/authorize?client_id=test-client" })
    );
}

#[test]
#[ignore = "porting: mcp oauth-browser not implemented"]
fn browser_launch_receives_the_discovered_authorization_url() {
    assert!(authorization_url_has_client_id(
        "http://127.0.0.1:1234/authorize?client_id=test-client&state=abc",
        "test-client"
    )
    .unwrap());
    assert!(!authorization_url_has_client_id(
        "http://127.0.0.1:1234/authorize?state=abc",
        "test-client"
    )
    .unwrap());
}
