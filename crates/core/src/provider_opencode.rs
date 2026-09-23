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

use crate::{CoreError, CoreResult};

/// The OpenCode provider plugin.
#[derive(Debug, Default)]
pub struct OpencodePlugin;

impl OpencodePlugin {
    /// Whether the plugin rewrites `provider_id` (only `opencode`).
    pub fn matches_provider(_provider_id: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::matches_provider",
        ))
    }

    /// Whether a model with the given input cost (per million tokens) is paid,
    /// and therefore requires credentials.
    pub fn requires_credentials(_input_cost: f64) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::requires_credentials",
        ))
    }

    /// Whether a model is enabled given whether credentials are available and
    /// its input cost.
    pub fn model_enabled(_credentials: bool, _input_cost: f64) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::model_enabled",
        ))
    }

    /// Resolve whether credentials are available: a primary env key, a
    /// configured env-method variable, or a configured `apiKey`.
    pub fn has_credentials(
        _primary_env_key: Option<&str>,
        _env_method_present: bool,
        _configured_api_key: Option<&str>,
    ) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::has_credentials",
        ))
    }

    /// The API key to write into the provider body: the configured key when
    /// authenticated, otherwise the public fallback.
    pub fn api_key(
        _credentials: bool,
        _configured_api_key: Option<&str>,
    ) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::api_key",
        ))
    }

    /// Choose the small model from candidate ids, preferring `gpt-5-nano`.
    pub fn small_model(_candidates: &[&str]) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_opencode::OpencodePlugin::small_model",
        ))
    }
}
