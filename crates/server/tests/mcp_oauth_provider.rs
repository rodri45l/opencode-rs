//! Port of packages/opencode/test/mcp/oauth-provider.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the OAuth redirect URL precedence (redirectUri >
//! callbackPort > default) and the client metadata advertised to the
//! authorization server. The `determineScope` cases exercise the MCP SDK's
//! own scope selection, not this codebase, so they are dropped.

use opencode_server::port::mcp::{
    McpOAuthConfig, McpOAuthProvider, OAUTH_CALLBACK_PATH, OAUTH_CALLBACK_PORT,
};

fn provider(config: McpOAuthConfig) -> McpOAuthProvider {
    McpOAuthProvider::new("test-server", "https://mcp.example.com/mcp", config)
}

#[test]
fn redirect_url_defaults_to_the_local_callback() {
    let provider = provider(McpOAuthConfig::default());
    assert_eq!(
        provider.redirect_url(),
        format!("http://127.0.0.1:{OAUTH_CALLBACK_PORT}{OAUTH_CALLBACK_PATH}")
    );
}

#[test]
fn redirect_url_uses_callback_port_when_set() {
    let provider = provider(McpOAuthConfig {
        callback_port: Some(6620),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.redirect_url(),
        format!("http://127.0.0.1:6620{OAUTH_CALLBACK_PATH}")
    );
}

#[test]
fn redirect_uri_takes_precedence_over_callback_port() {
    let provider = provider(McpOAuthConfig {
        callback_port: Some(6620),
        redirect_uri: Some("http://127.0.0.1:9999/custom/callback".to_string()),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.redirect_url(),
        "http://127.0.0.1:9999/custom/callback"
    );
}

#[test]
fn redirect_uri_is_used_when_set_without_callback_port() {
    let provider = provider(McpOAuthConfig {
        redirect_uri: Some("http://127.0.0.1:8080/oauth/callback".to_string()),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.redirect_url(),
        "http://127.0.0.1:8080/oauth/callback"
    );
}

#[test]
fn client_metadata_includes_redirect_uris_from_redirect_url() {
    let provider = provider(McpOAuthConfig {
        callback_port: Some(6620),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.client_metadata().redirect_uris,
        [format!("http://127.0.0.1:6620{OAUTH_CALLBACK_PATH}")]
    );
}

#[test]
fn client_metadata_includes_scope_when_set() {
    let provider = provider(McpOAuthConfig {
        scope: Some("openid offline_access".to_string()),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.client_metadata().scope.as_deref(),
        Some("openid offline_access")
    );
}

#[test]
fn client_metadata_omits_scope_when_not_set() {
    let provider = provider(McpOAuthConfig::default());
    assert_eq!(provider.client_metadata().scope, None);
}

#[test]
fn token_endpoint_auth_method_is_client_secret_post_when_secret_provided() {
    let provider = provider(McpOAuthConfig {
        client_secret: Some("secret".to_string()),
        ..McpOAuthConfig::default()
    });
    assert_eq!(
        provider.client_metadata().token_endpoint_auth_method,
        "client_secret_post"
    );
}

#[test]
fn token_endpoint_auth_method_is_none_without_secret() {
    let provider = provider(McpOAuthConfig::default());
    assert_eq!(
        provider.client_metadata().token_endpoint_auth_method,
        "none"
    );
}
