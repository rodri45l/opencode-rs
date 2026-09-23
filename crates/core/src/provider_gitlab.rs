//! GitLab provider plugin option resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin/provider/gitlab.ts`:
//! the exact `gitlab-ai-provider` package only, an instance URL from the
//! configured option then `GITLAB_INSTANCE_URL` then `https://gitlab.com`, an
//! API key from the configured option then `GITLAB_TOKEN`, merged AI gateway
//! headers, and merged feature flags (configured flags win per key). The SDK
//! factory and workflow-model selection are dropped.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

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
    pub fn matches_package(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_gitlab::GitLabPlugin::matches_package",
        ))
    }

    /// Resolve the SDK options from the environment and configured options.
    pub fn resolve_options(
        _env: &BTreeMap<String, String>,
        _configured: &GitLabOptions,
    ) -> CoreResult<ResolvedGitLabOptions> {
        Err(CoreError::NotImplemented(
            "provider_gitlab::GitLabPlugin::resolve_options",
        ))
    }
}
