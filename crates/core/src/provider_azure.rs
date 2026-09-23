//! Azure provider plugin option resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin/provider/azure.ts`:
//! the SDK package is `@ai-sdk/azure`, a configured non-blank `resourceName`
//! wins over `AZURE_RESOURCE_NAME`, and a blank configured value falls back to
//! the environment. Language selection lives in [`crate::provider_sdk_plugins`].

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// The Azure provider plugin.
#[derive(Debug, Default)]
pub struct AzurePlugin;

impl AzurePlugin {
    /// Whether the plugin handles the exact `@ai-sdk/azure` package.
    pub fn matches_package(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_azure::AzurePlugin::matches_package",
        ))
    }

    /// Resolve the effective `resourceName` from the configured value and env.
    pub fn resolve_resource_name(
        _configured: Option<&str>,
        _env: &BTreeMap<String, String>,
    ) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_azure::AzurePlugin::resolve_resource_name",
        ))
    }
}
