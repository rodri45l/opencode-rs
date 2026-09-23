//! Plugin specifier parsing, first-party plugin hooks, and the workspace
//! adapter registry.
//!
//! Ports the observable behaviour of `packages/opencode/src/plugin/shared.ts`
//! (`parsePluginSpecifier`), the Cerebras/Cloudflare/GitHub-Copilot hook
//! shapes, the WebSocket rollout gate, and the plugin-provided workspace
//! adapter registration used by `plugin/workspace-adapter.test.ts`.

use std::collections::BTreeMap;

/// A parsed npm-style plugin specifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSpecifier {
    /// Package name (scoped names keep their `@scope/` prefix).
    pub pkg: String,
    /// Requested version, `latest` when omitted.
    pub version: String,
}

/// Parse an npm package specifier into its package name and version.
pub fn parse_plugin_specifier(spec: &str) -> PluginSpecifier {
    // Protocol URLs resolve to themselves with no version, matching
    // `npm-package-arg`'s URL results.
    if spec.starts_with("git+")
        || spec.starts_with("http://")
        || spec.starts_with("https://")
        || spec.starts_with("file:")
    {
        return PluginSpecifier {
            pkg: spec.to_string(),
            version: String::new(),
        };
    }

    if let Some(rest) = spec.strip_prefix("npm:") {
        let (pkg, version) = split_name_version(rest);
        return PluginSpecifier {
            pkg,
            version: or_latest(version),
        };
    }

    let (pkg, version) = split_name_version(spec);
    PluginSpecifier {
        pkg,
        version: or_latest(version),
    }
}

fn or_latest(version: &str) -> String {
    if version.is_empty() || version == "*" {
        "latest".to_string()
    } else {
        version.to_string()
    }
}

fn split_name_version(spec: &str) -> (String, &str) {
    // A leading `@` belongs to a scoped package name, so search for the
    // version separator after it.
    let search_from = if spec.starts_with('@') { 1 } else { 0 };
    match spec[search_from..].find('@') {
        Some(offset) => {
            let separator = search_from + offset;
            (spec[..separator].to_string(), &spec[separator + 1..])
        }
        None => (spec.to_string(), ""),
    }
}

/// Whether the experimental WebSocket transport is enabled for a channel:
/// releases must opt in, pre-release channels get it by default.
pub fn experimental_websockets_enabled(enabled: bool, channel: Option<&str>) -> bool {
    enabled || matches!(channel, Some("local" | "dev" | "beta"))
}

/// Chat parameters passed to a provider request.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatParams {
    /// Generic output cap.
    pub max_output_tokens: Option<u32>,
    /// Provider-specific options.
    pub options: serde_json::Map<String, serde_json::Value>,
}

/// Ports the Cerebras `chat.params` hook: when `max_completion_tokens` is
/// configured it replaces the generic output cap, which must be dropped.
pub fn cerebras_chat_params(npm: &str, params: &mut ChatParams) {
    if npm != "@ai-sdk/cerebras" {
        return;
    }
    if params.options.contains_key("max_completion_tokens") {
        params.max_output_tokens = None;
    }
}

/// A provider auth method advertised by a plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthMethod {
    /// Human label shown in the auth picker.
    pub label: String,
}

/// The subset of plugin hooks exercised by the Cloudflare AI Gateway test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareHooks {
    /// The provider this plugin supplies auth for.
    pub auth_provider: Option<String>,
    /// The auth methods it registers.
    pub auth_methods: Vec<AuthMethod>,
    /// Whether the plugin still exposes a `chat.params` hook.
    pub has_chat_params: bool,
}

/// The Cloudflare AI Gateway auth hook set.
pub fn cloudflare_ai_gateway_auth_plugin() -> CloudflareHooks {
    CloudflareHooks {
        auth_provider: Some("cloudflare-ai-gateway".to_string()),
        auth_methods: vec![AuthMethod {
            label: "Cloudflare AI Gateway".to_string(),
        }],
        has_chat_params: false,
    }
}

/// Insert the Copilot interaction header when the provider is Copilot; other
/// providers pass through untouched.
pub fn apply_copilot_interaction_headers(
    provider_id: &str,
    session_id: &str,
    headers: &mut BTreeMap<String, String>,
) {
    if !provider_id.contains("github-copilot") {
        return;
    }
    headers.insert("X-Interaction-Id".to_string(), session_id.to_string());
}

/// Input handed to a workspace adapter's `configure`.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceInput {
    /// Workspace name.
    pub name: String,
    /// Branch, when the adapter uses one.
    pub branch: Option<String>,
    /// Target directory.
    pub directory: String,
    /// Free-form adapter metadata.
    pub extra: serde_json::Value,
}

/// A plugin-provided workspace adapter.
pub struct WorkspaceAdapter {
    /// Human name.
    pub name: String,
    /// Human description.
    pub description: String,
    /// Derives the concrete workspace from the requested input.
    pub configure: Box<dyn Fn(WorkspaceInput) -> WorkspaceInput>,
}

/// A typed workspace registry failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
    /// No adapter was registered for the requested type.
    NotFound(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::NotFound(kind) => write!(f, "workspace adapter not found: {kind}"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

/// A registry of workspace adapters keyed by workspace type.
#[derive(Default)]
pub struct WorkspaceRegistry {
    adapters: std::collections::HashMap<String, WorkspaceAdapter>,
}

impl WorkspaceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an adapter under `kind`, replacing any previous one.
    pub fn register(&mut self, kind: &str, adapter: WorkspaceAdapter) {
        self.adapters.insert(kind.to_string(), adapter);
    }

    /// Configure a new workspace of `kind` from `input`.
    pub fn create(
        &self,
        kind: &str,
        input: WorkspaceInput,
    ) -> Result<WorkspaceInput, WorkspaceError> {
        let adapter = self
            .adapters
            .get(kind)
            .ok_or_else(|| WorkspaceError::NotFound(kind.to_string()))?;
        Ok((adapter.configure)(input))
    }
}
