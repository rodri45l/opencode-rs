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
use crate::CoreResult;

/// Default AWS region used when neither config nor environment supplies one.
pub const DEFAULT_REGION: &str = "us-east-1";

/// The Amazon Bedrock provider plugin.
#[derive(Debug, Default)]
pub struct AmazonBedrockPlugin;

impl AmazonBedrockPlugin {
    /// Whether the plugin handles `package`. It matches the exact Bedrock SDK
    /// package and its `mantle` subpath only.
    pub fn matches_package(package: &str) -> CoreResult<bool> {
        Ok(package == "@ai-sdk/amazon-bedrock" || package == "@ai-sdk/amazon-bedrock/mantle")
    }

    /// Resolve the SDK region from config, falling back to the environment and
    /// then [`DEFAULT_REGION`].
    pub fn resolve_region(
        config_region: Option<&str>,
        env_region: Option<&str>,
    ) -> CoreResult<String> {
        Ok(config_region
            .or(env_region)
            .unwrap_or(DEFAULT_REGION)
            .to_string())
    }

    /// Resolve the SDK base URL. An explicit `endpoint` wins over a configured
    /// `baseURL`; both win over the region default.
    pub fn resolve_base_url(
        endpoint: Option<&str>,
        base_url: Option<&str>,
        region: &str,
    ) -> CoreResult<String> {
        if let Some(endpoint) = endpoint.filter(|value| !value.is_empty()) {
            return Ok(endpoint.to_string());
        }
        if let Some(base_url) = base_url.filter(|value| !value.is_empty()) {
            return Ok(base_url.to_string());
        }
        Ok(format!("https://bedrock-runtime.{region}.amazonaws.com"))
    }

    /// Apply the legacy cross-region inference prefix to a language model id for
    /// `region`.
    pub fn apply_language_prefix(region: &str, model_id: &str) -> CoreResult<String> {
        Ok(resolve_model_id(model_id, Some(region)))
    }

    /// Select the Mantle SDK accessor for an API model id.
    pub fn select_mantle_accessor(api_id: &str) -> CoreResult<LanguageSelector> {
        Ok(
            if api_id == "openai.gpt-oss-safeguard-20b" || api_id == "openai.gpt-oss-safeguard-120b"
            {
                LanguageSelector::Chat
            } else {
                LanguageSelector::Responses
            },
        )
    }
}

const CROSS_REGION_PREFIXES: &[&str] = &["global.", "us.", "eu.", "jp.", "apac.", "au."];

fn resolve_model_id(model_id: &str, region: Option<&str>) -> String {
    if model_id.starts_with("arn:") {
        return model_id.to_string();
    }
    if CROSS_REGION_PREFIXES
        .iter()
        .any(|prefix| model_id.starts_with(prefix))
    {
        return model_id.to_string();
    }
    let resolved_region = region.unwrap_or(DEFAULT_REGION);
    let region_prefix = resolved_region.split('-').next().unwrap_or("");
    match region_prefix {
        "us" => {
            let requires_prefix = [
                "nova-micro",
                "nova-lite",
                "nova-pro",
                "nova-premier",
                "nova-2",
                "claude",
                "deepseek.r1",
            ]
            .iter()
            .any(|item| model_id.contains(item));
            if requires_prefix && !resolved_region.starts_with("us-gov") {
                format!("{region_prefix}.{model_id}")
            } else {
                model_id.to_string()
            }
        }
        "eu" => {
            let region_requires_prefix = [
                "eu-west-1",
                "eu-west-2",
                "eu-west-3",
                "eu-north-1",
                "eu-central-1",
                "eu-south-1",
                "eu-south-2",
            ]
            .iter()
            .any(|item| resolved_region.contains(item));
            let model_requires_prefix = ["claude", "nova-lite", "nova-micro", "llama3", "pixtral"]
                .iter()
                .any(|item| model_id.contains(item));
            if region_requires_prefix && model_requires_prefix {
                format!("{region_prefix}.{model_id}")
            } else {
                model_id.to_string()
            }
        }
        "ap" => {
            let australia = ["ap-southeast-2", "ap-southeast-4"].contains(&resolved_region);
            if australia
                && ["anthropic.claude-sonnet-4-5", "anthropic.claude-haiku"]
                    .iter()
                    .any(|item| model_id.contains(item))
            {
                return format!("au.{model_id}");
            }
            let prefix = if resolved_region == "ap-northeast-1" {
                "jp"
            } else {
                "apac"
            };
            if ["claude", "nova-lite", "nova-micro", "nova-pro"]
                .iter()
                .any(|item| model_id.contains(item))
            {
                format!("{prefix}.{model_id}")
            } else {
                model_id.to_string()
            }
        }
        _ => model_id.to_string(),
    }
}
