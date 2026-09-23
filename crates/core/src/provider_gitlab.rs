//! GitLab provider plugin option resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin/provider/gitlab.ts`:
//! the exact `gitlab-ai-provider` package only, an instance URL from the
//! configured option then `GITLAB_INSTANCE_URL` then `https://gitlab.com`, an
//! API key from the configured option then `GITLAB_TOKEN`, merged AI gateway
//! headers, and merged feature flags (configured flags win per key). The SDK
//! factory and workflow-model selection are dropped.

use std::collections::BTreeMap;

use crate::CoreResult;

/// Configured GitLab SDK options.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitLabOptions {
    /// Configured instance URL.
    pub instance_url: Option<String>,
    /// Configured API key.
    pub api_key: Option<String>,
    /// Configured AI gateway headers.
    pub ai_gateway_headers: BTreeMap<String, String>,
    /// Configured feature flags.
    pub feature_flags: BTreeMap<String, bool>,
}

/// Resolved GitLab SDK options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGitLabOptions {
    /// The effective instance URL.
    pub instance_url: String,
    /// The effective API key, if any.
    pub api_key: Option<String>,
    /// The effective AI gateway headers.
    pub ai_gateway_headers: BTreeMap<String, String>,
    /// The effective feature flags.
    pub feature_flags: BTreeMap<String, bool>,
}

/// The GitLab provider plugin.
#[derive(Debug, Default)]
pub struct GitLabPlugin;

impl GitLabPlugin {
    /// Whether the plugin handles the exact `gitlab-ai-provider` package.
    pub fn matches_package(package: &str) -> CoreResult<bool> {
        Ok(package == "gitlab-ai-provider")
    }

    /// Resolve the SDK options from the environment and configured options.
    pub fn resolve_options(
        env: &BTreeMap<String, String>,
        configured: &GitLabOptions,
    ) -> CoreResult<ResolvedGitLabOptions> {
        let instance_url = configured
            .instance_url
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| env.get("GITLAB_INSTANCE_URL").cloned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "https://gitlab.com".to_string());
        let api_key = configured
            .api_key
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| env.get("GITLAB_TOKEN").cloned())
            .filter(|value| !value.is_empty());

        let mut ai_gateway_headers = BTreeMap::new();
        ai_gateway_headers.insert(
            "User-Agent".to_string(),
            "opencode gitlab-ai-provider/0.0.0 (linux; unknown; unknown)".to_string(),
        );
        ai_gateway_headers.insert(
            "anthropic-beta".to_string(),
            "context-1m-2025-08-07".to_string(),
        );
        for (key, value) in &configured.ai_gateway_headers {
            ai_gateway_headers.insert(key.clone(), value.clone());
        }

        let mut feature_flags = BTreeMap::new();
        feature_flags.insert("duo_agent_platform_agentic_chat".to_string(), true);
        feature_flags.insert("duo_agent_platform".to_string(), true);
        for (key, value) in &configured.feature_flags {
            feature_flags.insert(key.clone(), *value);
        }

        Ok(ResolvedGitLabOptions {
            instance_url,
            api_key,
            ai_gateway_headers,
            feature_flags,
        })
    }
}
