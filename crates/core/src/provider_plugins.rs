//! Built-in provider plugins.
//!
//! Ports the observable behaviour of the legacy provider plugins in
//! `packages/core/src/plugin/provider/*`: Kilo and Zenmux apply their historical
//! referer headers to exactly their own provider id, preserving configured
//! overrides, and the plugin registry orders `snowflake-cortex` before the
//! openai-compatible fallback.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// A provider request envelope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProviderRequest {
    /// Request headers.
    pub headers: BTreeMap<String, String>,
    /// Request body.
    pub body: serde_json::Value,
}

/// The built-in provider plugin registry.
#[derive(Debug, Default)]
pub struct ProviderPlugins;

impl ProviderPlugins {
    /// Registered plugin ids in precedence order.
    pub fn ids() -> Vec<&'static str> {
        vec![
            "google-vertex",
            "snowflake-cortex",
            "kilo",
            "zenmux",
            "openai-compatible",
        ]
    }

    /// Apply the Kilo legacy referer headers to `request` for `provider_id`.
    pub fn apply_kilo(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_plugins::ProviderPlugins::apply_kilo",
        ))
    }

    /// Apply the Zenmux legacy referer headers to `request` for `provider_id`.
    pub fn apply_zenmux(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_plugins::ProviderPlugins::apply_zenmux",
        ))
    }
}
