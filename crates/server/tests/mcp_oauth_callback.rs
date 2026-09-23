//! Port of packages/opencode/test/mcp/oauth-callback.test.ts (upstream 18ef3cc).
//!
//! Subset: `parseRedirectUri` and the callback page HTML escaping are ported
//! against `opencode_server::mcp_oauth`. The loopback callback server lifecycle
//! (`ensureRunning`/`waitForCallback`/`stop`, IPv4-only binding) needs the MCP
//! OAuth runtime and is left to the MCP phase.

use opencode_server::mcp_oauth::{
    error_page, escape_html, parse_redirect_uri, success_page, OAUTH_CALLBACK_PATH,
    OAUTH_CALLBACK_PORT,
};

#[test]
fn parse_redirect_uri_returns_defaults_when_no_uri_provided() {
    let result = parse_redirect_uri(None);
    assert_eq!(result.port, 19876);
    assert_eq!(result.path, "/mcp/oauth/callback");
}

#[test]
fn parse_redirect_uri_parses_port_and_path_from_uri() {
    let result = parse_redirect_uri(Some("http://127.0.0.1:8080/oauth/callback"));
    assert_eq!(result.port, 8080);
    assert_eq!(result.path, "/oauth/callback");
}

#[test]
fn parse_redirect_uri_returns_defaults_for_invalid_uri() {
    let result = parse_redirect_uri(Some("not-a-valid-url"));
    assert_eq!(result.port, OAUTH_CALLBACK_PORT);
    assert_eq!(result.path, OAUTH_CALLBACK_PATH);
}

#[test]
fn parse_redirect_uri_defaults_port_from_scheme() {
    assert_eq!(parse_redirect_uri(Some("https://example.com/cb")).port, 443);
    assert_eq!(parse_redirect_uri(Some("http://example.com/cb")).port, 80);
}

#[test]
fn escape_html_encodes_markup_metacharacters() {
    assert_eq!(
        escape_html("<script>alert(\"xss\" & 'more')</script>"),
        "&lt;script&gt;alert(&quot;xss&quot; &amp; &#39;more&#39;)&lt;/script&gt;"
    );
}

#[test]
fn callback_page_escapes_provider_error_markup() {
    let error = "<script>alert(\"xss\" & 'more')</script>";
    let body = error_page(error, Some("MCP"));
    assert!(
        body.contains("&lt;script&gt;alert(&quot;xss&quot; &amp; &#39;more&#39;)&lt;/script&gt;")
    );
    assert!(!body.contains(error));
}

#[test]
fn callback_page_keeps_normal_provider_errors_readable() {
    let body = error_page("The user denied access", Some("MCP"));
    assert!(body.contains(r#"<pre class="detail" id="oc-detail">The user denied access</pre>"#));
}

#[test]
fn success_page_names_the_provider() {
    let body = success_page(Some("MCP"));
    assert!(body.contains("Authorization successful"));
    assert!(body.contains("OpenCode is now connected to MCP."));
}
