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
use crate::CoreResult;

/// Endpoint prefix for the Workers AI OpenAI-compatible API.
pub const ACCOUNT_ENDPOINT_PREFIX: &str = "https://api.cloudflare.com/client/v4/accounts";

/// The Cloudflare Workers AI provider plugin.
#[derive(Debug, Default)]
pub struct CloudflareWorkersAiPlugin;

impl CloudflareWorkersAiPlugin {
    /// Whether the plugin handles `package` (the OpenAI-compatible SDK only).
    pub fn matches_package(package: &str) -> CoreResult<bool> {
        Ok(package == "@ai-sdk/openai-compatible")
    }

    /// Resolve the endpoint URL, if any. An explicit configured URL is kept
    /// verbatim; otherwise an endpoint is derived from the account id (the
    /// environment id wins over the configured one). Returns `None` when neither
    /// a URL nor an account id is available.
    pub fn resolve_endpoint(
        configured_url: Option<&str>,
        env_account_id: Option<&str>,
        configured_account_id: Option<&str>,
    ) -> CoreResult<Option<String>> {
        if let Some(url) = configured_url.filter(|value| !value.is_empty()) {
            return Ok(Some(Self::expand_account_id(
                url,
                env_account_id.or(configured_account_id),
            )?));
        }
        let account_id = env_account_id
            .filter(|value| !value.is_empty())
            .or_else(|| configured_account_id.filter(|value| !value.is_empty()));
        let Some(account_id) = account_id else {
            return Ok(None);
        };
        Ok(Some(format!(
            "{ACCOUNT_ENDPOINT_PREFIX}/{account_id}/ai/v1"
        )))
    }

    /// Expand `${CLOUDFLARE_ACCOUNT_ID}` in `template` using `account_id`.
    pub fn expand_account_id(template: &str, account_id: Option<&str>) -> CoreResult<String> {
        match account_id {
            Some(account_id) => Ok(template.replace("${CLOUDFLARE_ACCOUNT_ID}", account_id)),
            None => Ok(template.to_string()),
        }
    }

    /// Resolve the Authorization header value, preferring the environment key
    /// over the auth key over the configured key.
    pub fn authorization(
        env_api_key: Option<&str>,
        configured_api_key: Option<&str>,
        auth_api_key: Option<&str>,
    ) -> CoreResult<Option<String>> {
        let key = env_api_key
            .filter(|value| !value.is_empty())
            .or_else(|| auth_api_key.filter(|value| !value.is_empty()))
            .or_else(|| configured_api_key.filter(|value| !value.is_empty()));
        Ok(key.map(|key| format!("Bearer {key}")))
    }

    /// Merge the custom request headers with the Cloudflare authorization.
    pub fn merge_headers(
        authorization: Option<&str>,
        custom: &BTreeMap<String, String>,
    ) -> CoreResult<BTreeMap<String, String>> {
        let mut merged = custom.clone();
        if let Some(authorization) = authorization {
            merged.insert("authorization".to_string(), authorization.to_string());
        }
        Ok(merged)
    }

    /// Select the language model accessor with the API model id.
    pub fn select_language(_api_id: &str) -> CoreResult<LanguageSelector> {
        Ok(LanguageSelector::LanguageModel)
    }
}
