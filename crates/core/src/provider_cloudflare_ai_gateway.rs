//! Cloudflare AI Gateway provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/cloudflare-ai-gateway.ts`: the plugin
//! binds to `ai-gateway-provider` only, resolves the account/gateway/token
//! triple (environment values win over auth/config-derived options; the token
//! falls back to `CF_AIG_TOKEN`; a `gatewayId` option is accepted as `gateway`),
//! returns no SDK when any part is missing or a `baseURL` is configured, and
//! passes legacy metadata/cache/log options under the `options` key. The
//! `AISDK`/`Catalog`/`PluginHost` wiring and the SDK factory are dropped.

use serde_json::Value;

use crate::CoreResult;

/// The resolved Cloudflare AI Gateway SDK configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareGatewayConfig {
    /// Cloudflare account id.
    pub account_id: String,
    /// AI Gateway id.
    pub gateway: String,
    /// API token.
    pub api_key: String,
}

/// The Cloudflare environment values relevant to the plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CloudflareGatewayEnv {
    /// `CLOUDFLARE_ACCOUNT_ID`.
    pub account_id: Option<String>,
    /// `CLOUDFLARE_GATEWAY_ID`.
    pub gateway: Option<String>,
    /// `CLOUDFLARE_API_TOKEN`.
    pub api_token: Option<String>,
    /// `CF_AIG_TOKEN`.
    pub cf_aig_token: Option<String>,
}

/// Auth/config-derived options copied into provider options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CloudflareGatewayOptions {
    /// Configured account id.
    pub account_id: Option<String>,
    /// Configured gateway id.
    pub gateway: Option<String>,
    /// Configured `gatewayId` alias.
    pub gateway_id: Option<String>,
    /// Configured API key.
    pub api_key: Option<String>,
}

/// The Cloudflare AI Gateway provider plugin.
#[derive(Debug, Default)]
pub struct CloudflareAiGatewayPlugin;

impl CloudflareAiGatewayPlugin {
    /// Whether the plugin handles `package` (the AI Gateway SDK only).
    pub fn matches_package(package: &str) -> CoreResult<bool> {
        Ok(package == "ai-gateway-provider")
    }

    /// Resolve the SDK configuration, or `None` when account, gateway, or token
    /// is absent.
    pub fn resolve_config(
        env: &CloudflareGatewayEnv,
        options: &CloudflareGatewayOptions,
    ) -> CoreResult<Option<CloudflareGatewayConfig>> {
        let account_id = env
            .account_id
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| options.account_id.clone().filter(|value| !value.is_empty()));
        let gateway = env
            .gateway
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| options.gateway.clone().filter(|value| !value.is_empty()))
            .or_else(|| options.gateway_id.clone().filter(|value| !value.is_empty()));
        let api_key = env
            .api_token
            .clone()
            .filter(|value| !value.is_empty())
            .or_else(|| env.cf_aig_token.clone().filter(|value| !value.is_empty()))
            .or_else(|| options.api_key.clone().filter(|value| !value.is_empty()));

        match (account_id, gateway, api_key) {
            (Some(account_id), Some(gateway), Some(api_key)) => Ok(Some(CloudflareGatewayConfig {
                account_id,
                gateway,
                api_key,
            })),
            _ => Ok(None),
        }
    }

    /// Resolve the AI Gateway metadata: an explicit `metadata` option wins over
    /// the legacy `cf-aig-metadata` JSON header.
    pub fn resolve_metadata(
        metadata: Option<&Value>,
        cf_aig_metadata_header: Option<&str>,
    ) -> CoreResult<Option<Value>> {
        if let Some(metadata) = metadata.filter(|value| !value.is_null()) {
            return Ok(Some(metadata.clone()));
        }
        if let Some(header) = cf_aig_metadata_header {
            if let Ok(parsed) = serde_json::from_str::<Value>(header) {
                return Ok(Some(parsed));
            }
        }
        Ok(None)
    }
}
