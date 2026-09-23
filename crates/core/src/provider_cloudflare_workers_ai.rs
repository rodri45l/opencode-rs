//! Cloudflare Workers AI provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/cloudflare-workers-ai.ts`: the plugin
//! binds only to `@ai-sdk/openai-compatible`, resolves the account endpoint URL
//! (explicit configured URL wins; otherwise derived from the environment account
//! id, which wins over a configured `accountId`), expands `${CLOUDFLARE_ACCOUNT_ID}`
//! inside endpoint URLs, prefers the environment API key over auth/config keys,
//! and selects `languageModel` with the API model id. The `AISDK`/`Catalog`/
//! `PluginHost` wiring and the SDK factory are dropped.

use std::collections::BTreeMap;

use crate::provider_sdk_plugins::LanguageSelector;
use crate::{CoreError, CoreResult};

/// Endpoint prefix for the Workers AI OpenAI-compatible API.
pub const ACCOUNT_ENDPOINT_PREFIX: &str = "https://api.cloudflare.com/client/v4/accounts";

/// The Cloudflare Workers AI provider plugin.
#[derive(Debug, Default)]
pub struct CloudflareWorkersAiPlugin;

impl CloudflareWorkersAiPlugin {
    /// Whether the plugin handles `package` (the OpenAI-compatible SDK only).
    pub fn matches_package(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::matches_package",
        ))
    }

    /// Resolve the endpoint URL, if any. An explicit configured URL is kept
    /// verbatim; otherwise an endpoint is derived from the account id (the
    /// environment id wins over the configured one). Returns `None` when neither
    /// a URL nor an account id is available.
    pub fn resolve_endpoint(
        _configured_url: Option<&str>,
        _env_account_id: Option<&str>,
        _configured_account_id: Option<&str>,
    ) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::resolve_endpoint",
        ))
    }

    /// Expand `${CLOUDFLARE_ACCOUNT_ID}` in `template` using `account_id`.
    pub fn expand_account_id(_template: &str, _account_id: Option<&str>) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::expand_account_id",
        ))
    }

    /// Resolve the Authorization header value, preferring the environment key
    /// over the auth key over the configured key.
    pub fn authorization(
        _env_api_key: Option<&str>,
        _configured_api_key: Option<&str>,
        _auth_api_key: Option<&str>,
    ) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::authorization",
        ))
    }

    /// Merge the custom request headers with the Cloudflare authorization.
    pub fn merge_headers(
        _authorization: Option<&str>,
        _custom: &BTreeMap<String, String>,
    ) -> CoreResult<BTreeMap<String, String>> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::merge_headers",
        ))
    }

    /// Select the language model accessor with the API model id.
    pub fn select_language(_api_id: &str) -> CoreResult<LanguageSelector> {
        Err(CoreError::NotImplemented(
            "provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin::select_language",
        ))
    }
}
