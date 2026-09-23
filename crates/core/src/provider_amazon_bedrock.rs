//! Amazon Bedrock provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/amazon-bedrock.ts`: the plugin binds to
//! the exact `@ai-sdk/amazon-bedrock` package (and its `mantle` subpath), the
//! legacy cross-region inference prefix matrix applied to language model ids,
//! region resolution (`config` over `AWS_REGION` over `us-east-1`), base-URL
//! resolution (`endpoint` over `baseURL` over the region default), and Mantle
//! accessor selection (`responses` for GPT-5, `chat` for GPT-OSS). The
//! `AISDK`/`Catalog`/`PluginHost` wiring, SigV4 fetch signing, and environment
//! credential loading are dropped.

use crate::provider_sdk_plugins::LanguageSelector;
use crate::{CoreError, CoreResult};

/// Default AWS region used when neither config nor environment supplies one.
pub const DEFAULT_REGION: &str = "us-east-1";

/// The Amazon Bedrock provider plugin.
#[derive(Debug, Default)]
pub struct AmazonBedrockPlugin;

impl AmazonBedrockPlugin {
    /// Whether the plugin handles `package`. It matches the exact Bedrock SDK
    /// package and its `mantle` subpath only.
    pub fn matches_package(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_amazon_bedrock::AmazonBedrockPlugin::matches_package",
        ))
    }

    /// Resolve the SDK region from config, falling back to the environment and
    /// then [`DEFAULT_REGION`].
    pub fn resolve_region(
        _config_region: Option<&str>,
        _env_region: Option<&str>,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_amazon_bedrock::AmazonBedrockPlugin::resolve_region",
        ))
    }

    /// Resolve the SDK base URL. An explicit `endpoint` wins over a configured
    /// `baseURL`; both win over the region default.
    pub fn resolve_base_url(
        _endpoint: Option<&str>,
        _base_url: Option<&str>,
        _region: &str,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_amazon_bedrock::AmazonBedrockPlugin::resolve_base_url",
        ))
    }

    /// Apply the legacy cross-region inference prefix to a language model id for
    /// `region`.
    pub fn apply_language_prefix(_region: &str, _model_id: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_amazon_bedrock::AmazonBedrockPlugin::apply_language_prefix",
        ))
    }

    /// Select the Mantle SDK accessor for an API model id.
    pub fn select_mantle_accessor(_api_id: &str) -> CoreResult<LanguageSelector> {
        Err(CoreError::NotImplemented(
            "provider_amazon_bedrock::AmazonBedrockPlugin::select_mantle_accessor",
        ))
    }
}
