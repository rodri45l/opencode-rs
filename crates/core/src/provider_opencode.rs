//! OpenCode provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/opencode.ts`: the plugin only rewrites the
//! `opencode` provider, falls back to the public API key when no credential is
//! available, disables paid models without credentials (free and output-only
//! models stay enabled), enables every model once a credential exists (from
//! `OPENCODE_API_KEY`, a configured env method, or a configured `apiKey`), and
//! prefers `gpt-5-nano` as the small model. The `Catalog`/`Credential`/
//! `Integration`/`PluginHost` wiring and the remote provider fetch are dropped.

use crate::CoreResult;

/// The OpenCode provider plugin.
#[derive(Debug, Default)]
pub struct OpencodePlugin;

impl OpencodePlugin {
    /// Whether the plugin rewrites `provider_id` (only `opencode`).
    pub fn matches_provider(provider_id: &str) -> CoreResult<bool> {
        Ok(provider_id == "opencode")
    }

    /// Whether a model with the given input cost (per million tokens) is paid,
    /// and therefore requires credentials.
    pub fn requires_credentials(input_cost: f64) -> CoreResult<bool> {
        Ok(input_cost > 0.0)
    }

    /// Whether a model is enabled given whether credentials are available and
    /// its input cost.
    pub fn model_enabled(credentials: bool, input_cost: f64) -> CoreResult<bool> {
        Ok(credentials || !Self::requires_credentials(input_cost)?)
    }

    /// Resolve whether credentials are available: a primary env key, a
    /// configured env-method variable, or a configured `apiKey`.
    pub fn has_credentials(
        primary_env_key: Option<&str>,
        env_method_present: bool,
        configured_api_key: Option<&str>,
    ) -> CoreResult<bool> {
        Ok(primary_env_key.is_some_and(|value| !value.is_empty())
            || env_method_present
            || configured_api_key.is_some_and(|value| !value.is_empty()))
    }

    /// The API key to write into the provider body: the configured key when
    /// authenticated, otherwise the public fallback.
    pub fn api_key(
        credentials: bool,
        configured_api_key: Option<&str>,
    ) -> CoreResult<Option<String>> {
        if credentials {
            Ok(configured_api_key
                .filter(|value| !value.is_empty())
                .map(str::to_string))
        } else {
            Ok(Some("public".to_string()))
        }
    }

    /// Choose the small model from candidate ids, preferring `gpt-5-nano`.
    pub fn small_model(candidates: &[&str]) -> CoreResult<Option<String>> {
        if let Some(nano) = candidates.iter().find(|id| **id == "gpt-5-nano") {
            return Ok(Some((*nano).to_string()));
        }
        Ok(candidates.first().map(|id| (*id).to_string()))
    }
}
