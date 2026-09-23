//! Snowflake Cortex provider plugin.
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/snowflake-cortex.ts`: tokens resolve from
//! `SNOWFLAKE_CORTEX_PAT`/`SNOWFLAKE_CORTEX_TOKEN` with an options fallback,
//! `max_tokens` is rewritten to `max_completion_tokens`, a `400` "Conversation
//! complete" body settles as a stop response, streaming chunks rewrite an empty
//! role to `assistant`, and the plugin is registered before the
//! openai-compatible fallback.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// Snowflake Cortex SDK options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CortexOptions {
    /// PAT fallback.
    pub pat: Option<String>,
    /// OAuth token fallback.
    pub token: Option<String>,
    /// API key fallback.
    pub api_key: Option<String>,
}

/// The Snowflake Cortex provider plugin.
#[derive(Debug, Default)]
pub struct SnowflakeCortexPlugin;

impl SnowflakeCortexPlugin {
    /// The registry index of a built-in provider plugin id.
    pub fn registry_index(_id: &str) -> CoreResult<usize> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::registry_index",
        ))
    }

    /// Resolve the bearer token from the environment, then options.
    pub fn resolve_token(
        _env: &BTreeMap<String, String>,
        _options: &CortexOptions,
    ) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::resolve_token",
        ))
    }

    /// Whether usage is requested on the SDK options.
    pub fn include_usage(
        _env: &BTreeMap<String, String>,
        _options: &CortexOptions,
    ) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::include_usage",
        ))
    }

    /// Rewrite `max_tokens` to `max_completion_tokens`, preserving other bodies.
    pub fn rewrite_request_body(_body: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::rewrite_request_body",
        ))
    }

    /// Normalize a Cortex response: a `400` "Conversation complete" becomes a
    /// `200` stop response.
    pub fn rewrite_response(_status: u16, _body: &str) -> CoreResult<(u16, String)> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::rewrite_response",
        ))
    }

    /// Rewrite an empty streaming `role` to `assistant`.
    pub fn rewrite_streaming_role(_chunk: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_snowflake_cortex::SnowflakeCortexPlugin::rewrite_streaming_role",
        ))
    }
}
