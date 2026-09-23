//! MCP OAuth provider metadata, the serialized auth store, tool conversion,
//! and the session-recovery plan.
//!
//! Ports the observable behaviour of `packages/opencode/src/mcp/oauth-provider.ts`,
//! `packages/opencode/src/mcp/auth.ts`, `packages/opencode/src/mcp/catalog.ts`,
//! and the session-bound 404 retry from `mcp/session-recovery.test.ts`.

use serde_json::Value;

/// Default OAuth callback port.
pub const OAUTH_CALLBACK_PORT: u16 = 19876;
/// Default OAuth callback path.
pub const OAUTH_CALLBACK_PATH: &str = "/mcp/oauth/callback";

/// User-supplied OAuth configuration for an MCP server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpOAuthConfig {
    /// Explicit client id.
    pub client_id: Option<String>,
    /// Explicit client secret.
    pub client_secret: Option<String>,
    /// Requested scopes.
    pub scope: Option<String>,
    /// Callback port override.
    pub callback_port: Option<u16>,
    /// Full redirect URI override.
    pub redirect_uri: Option<String>,
}

/// The client metadata advertised to an authorization server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMetadata {
    /// Redirect URIs.
    pub redirect_uris: Vec<String>,
    /// Client display name.
    pub client_name: String,
    /// Client homepage.
    pub client_uri: String,
    /// Supported grant types.
    pub grant_types: Vec<String>,
    /// Supported response types.
    pub response_types: Vec<String>,
    /// Token endpoint auth method.
    pub token_endpoint_auth_method: String,
    /// Requested scopes, omitted when unset.
    pub scope: Option<String>,
}

/// An MCP OAuth client provider; only its synchronous getters are modelled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpOAuthProvider {
    /// MCP server name.
    pub name: String,
    /// Server URL.
    pub server_url: String,
    /// Resolved config.
    pub config: McpOAuthConfig,
}

impl McpOAuthProvider {
    /// Create a provider for `name`/`server_url`.
    pub fn new(
        name: impl Into<String>,
        server_url: impl Into<String>,
        config: McpOAuthConfig,
    ) -> Self {
        Self {
            name: name.into(),
            server_url: server_url.into(),
            config,
        }
    }

    /// The OAuth redirect URL.
    pub fn redirect_url(&self) -> String {
        if let Some(uri) = &self.config.redirect_uri {
            return uri.clone();
        }
        let port = self.config.callback_port.unwrap_or(OAUTH_CALLBACK_PORT);
        format!("http://127.0.0.1:{port}{OAUTH_CALLBACK_PATH}")
    }

    /// The client metadata advertised to the authorization server.
    pub fn client_metadata(&self) -> ClientMetadata {
        ClientMetadata {
            redirect_uris: vec![self.redirect_url()],
            client_name: "OpenCode".to_string(),
            client_uri: "https://opencode.ai".to_string(),
            grant_types: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            response_types: vec!["code".to_string()],
            token_endpoint_auth_method: if self.config.client_secret.is_some() {
                "client_secret_post".to_string()
            } else {
                "none".to_string()
            },
            scope: self.config.scope.clone(),
        }
    }
}

/// Stored OAuth tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct OAuthTokens {
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: Option<String>,
    /// Expiry as a Unix timestamp in seconds.
    pub expires_at: Option<f64>,
    /// Granted scopes.
    pub scope: Option<String>,
}

/// Dynamically-registered client information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInfo {
    /// Client id.
    pub client_id: String,
    /// Client secret.
    pub client_secret: Option<String>,
}

/// A serialized MCP auth entry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuthEntry {
    /// Stored tokens.
    pub tokens: Option<OAuthTokens>,
    /// Stored client information.
    pub client_info: Option<ClientInfo>,
    /// The server URL the entry was saved for.
    pub server_url: Option<String>,
}

/// The MCP auth store, merged per server name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct McpAuth {
    entries: std::collections::HashMap<String, AuthEntry>,
}

impl McpAuth {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update tokens for `name`.
    pub fn update_tokens(&mut self, name: &str, tokens: OAuthTokens, server_url: &str) {
        let entry = self.entries.entry(name.to_string()).or_default();
        entry.tokens = Some(tokens);
        entry.server_url = Some(server_url.to_string());
    }

    /// Insert or update client information for `name`.
    pub fn update_client_info(&mut self, name: &str, info: ClientInfo, server_url: &str) {
        let entry = self.entries.entry(name.to_string()).or_default();
        entry.client_info = Some(info);
        entry.server_url = Some(server_url.to_string());
    }

    /// Look up an entry by server name.
    pub fn get(&self, name: &str) -> Option<&AuthEntry> {
        self.entries.get(name)
    }
}

/// An MCP tool definition.
#[derive(Debug, Clone, PartialEq)]
pub struct McpToolDef {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON input schema.
    pub input_schema: Value,
}

/// A raw MCP tool-call result.
#[derive(Debug, Clone, PartialEq)]
pub struct McpToolCallResult {
    /// Content blocks.
    pub content: Vec<Value>,
    /// Structured content, when supplied.
    pub structured_content: Option<Value>,
}

/// Convert an MCP tool result, preserving content and falling back to a text
/// block when content is empty but structured content is present.
pub fn convert_tool_result(result: McpToolCallResult) -> McpToolCallResult {
    if !result.content.is_empty() {
        return result;
    }
    match result.structured_content.clone() {
        Some(structured) => McpToolCallResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": serde_json::to_string(&structured).unwrap_or_default(),
            })],
            structured_content: Some(structured),
        },
        None => result,
    }
}

/// A single MCP request in the session-recovery plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRequest {
    /// JSON-RPC method.
    pub method: String,
    /// Session id sent with the request, `None` when absent.
    pub session: Option<String>,
}

/// Build the request sequence used to recover from a session-bound 404:
/// reinitialize once, then retry the original request under the replacement
/// session.
pub fn session_recovery_plan(initial: &str, replacement: &str) -> Vec<RecoveryRequest> {
    let request = |method: &str, session: Option<&str>| RecoveryRequest {
        method: method.to_string(),
        session: session.map(str::to_string),
    };
    vec![
        request("initialize", None),
        request("notifications/initialized", Some(initial)),
        request("ping", Some(initial)),
        request("initialize", None),
        request("notifications/initialized", Some(replacement)),
        request("ping", Some(replacement)),
    ]
}
