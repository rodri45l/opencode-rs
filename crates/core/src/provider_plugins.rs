//! Built-in provider plugins.
//!
//! Ports the observable behaviour of the legacy provider plugins in
//! `packages/core/src/plugin/provider/*`: Kilo and Zenmux apply their historical
//! referer headers to exactly their own provider id, preserving configured
//! overrides, and the plugin registry orders `snowflake-cortex` before the
//! openai-compatible fallback.

use std::collections::BTreeMap;

use crate::CoreResult;

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
    pub fn apply_kilo(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "kilo" {
            return Ok(());
        }
        request.headers.insert(
            "HTTP-Referer".to_string(),
            "https://opencode.ai/".to_string(),
        );
        request
            .headers
            .insert("X-Title".to_string(), "opencode".to_string());
        Ok(())
    }

    /// Apply the Zenmux legacy referer headers to `request` for `provider_id`.
    pub fn apply_zenmux(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "zenmux" {
            return Ok(());
        }
        request
            .headers
            .entry("HTTP-Referer".to_string())
            .or_insert_with(|| "https://opencode.ai/".to_string());
        request
            .headers
            .entry("X-Title".to_string())
            .or_insert_with(|| "opencode".to_string());
        Ok(())
    }
}
