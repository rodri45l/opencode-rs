//! Azure provider plugin option resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin/provider/azure.ts`:
//! the SDK package is `@ai-sdk/azure`, a configured non-blank `resourceName`
//! wins over `AZURE_RESOURCE_NAME`, and a blank configured value falls back to
//! the environment. Language selection lives in [`crate::provider_sdk_plugins`].

use std::collections::BTreeMap;

use crate::CoreResult;

/// The Azure provider plugin.
#[derive(Debug, Default)]
pub struct AzurePlugin;

impl AzurePlugin {
    /// Whether the plugin handles the exact `@ai-sdk/azure` package.
    pub fn matches_package(package: &str) -> CoreResult<bool> {
        Ok(package == "@ai-sdk/azure")
    }

    /// Resolve the effective `resourceName` from the configured value and env.
    pub fn resolve_resource_name(
        configured: Option<&str>,
        env: &BTreeMap<String, String>,
    ) -> CoreResult<Option<String>> {
        if let Some(value) = configured.filter(|value| !value.trim().is_empty()) {
            return Ok(Some(value.to_string()));
        }
        Ok(env.get("AZURE_RESOURCE_NAME").cloned())
    }
}
