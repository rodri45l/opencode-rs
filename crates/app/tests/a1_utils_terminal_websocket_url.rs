//! Port of packages/app/src/utils/terminal-websocket-url.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::terminal_websocket_url::{terminal_websocket_url, Protocol, WsInput};
use opencode_test_support as ts;

fn base(protocol: Protocol) -> WsInput {
    WsInput {
        protocol,
        url: "http://127.0.0.1:49365".into(),
        id: "pty_test".into(),
        directory: "/tmp/project".into(),
        cursor: 0,
        same_origin: false,
        username: None,
        password: None,
        auth_token: false,
        ticket: None,
    }
}

#[test]
fn uses_the_current_ticketed_pty_route() {
    let mut input = base(Protocol::V2);
    input.ticket = Some("connect-ticket".into());
    let url = terminal_websocket_url(input);

    assert_eq!(url.protocol, "ws:");
    assert_eq!(url.username, "");
    assert_eq!(url.password, "");
    assert_eq!(url.pathname, "/api/pty/pty_test/connect");
    assert_eq!(
        url.params.get("location[directory]").map(String::as_str),
        Some("/tmp/project")
    );
    assert_eq!(url.params.get("cursor").map(String::as_str), Some("0"));
    assert_eq!(
        url.params.get("ticket").map(String::as_str),
        Some("connect-ticket")
    );
    assert!(!url.params.contains_key("auth_token"));
}

#[test]
fn uses_query_auth_without_embedding_credentials_for_v1() {
    let mut input = base(Protocol::V1);
    input.username = Some("opencode".into());
    input.password = Some("secret".into());
    let url = terminal_websocket_url(input);

    assert_eq!(url.protocol, "ws:");
    assert_eq!(url.username, "");
    assert_eq!(url.password, "");
    assert_eq!(url.pathname, "/pty/pty_test/connect");
    assert_eq!(
        url.params.get("directory").map(String::as_str),
        Some("/tmp/project")
    );
    assert_eq!(
        url.params.get("auth_token").map(String::as_str),
        Some(ts::encode_b64url(b"opencode:secret").as_str())
    );
}

#[test]
fn omits_query_auth_for_same_origin_saved_credentials_for_v1() {
    let mut input = base(Protocol::V1);
    input.url = "https://app.example.test".into();
    input.cursor = 10;
    input.same_origin = true;
    input.username = Some("opencode".into());
    input.password = Some("secret".into());
    let url = terminal_websocket_url(input);

    assert_eq!(url.protocol, "wss:");
    assert_eq!(url.pathname, "/pty/pty_test/connect");
    assert_eq!(
        url.params.get("directory").map(String::as_str),
        Some("/tmp/project")
    );
    assert!(!url.params.contains_key("auth_token"));
}

#[test]
fn uses_query_auth_for_same_origin_credentials_from_auth_token_for_v1() {
    let mut input = base(Protocol::V1);
    input.url = "https://app.example.test".into();
    input.cursor = 10;
    input.same_origin = true;
    input.username = Some("opencode".into());
    input.password = Some("secret".into());
    input.auth_token = true;
    let url = terminal_websocket_url(input);

    assert_eq!(url.protocol, "wss:");
    assert_eq!(url.pathname, "/pty/pty_test/connect");
    assert_eq!(
        url.params.get("directory").map(String::as_str),
        Some("/tmp/project")
    );
    assert_eq!(
        url.params.get("auth_token").map(String::as_str),
        Some(ts::encode_b64url(b"opencode:secret").as_str())
    );
}
