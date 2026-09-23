//! Legacy provider header plugins (re-derived behavioural subset).
//!
//! Ports the observable behaviour of the legacy referer/tracking-header
//! providers in `packages/core/src/plugin/provider/*`: each applies a fixed set
//! of request headers to exactly its own provider id, merges with configured
//! headers, and (for llmgateway) is guarded by the enabled flag. The
//! `Catalog`/`PluginHost` wiring is replaced by a pure request transform.

use crate::provider_plugins::ProviderRequest;
use crate::CoreResult;

/// The legacy provider header plugins.
#[derive(Debug, Default)]
pub struct ProviderHeaderPlugins;

impl ProviderHeaderPlugins {
    /// Registered legacy header plugin ids in precedence order.
    pub fn registered_ids() -> CoreResult<Vec<&'static str>> {
        Ok(vec![
            "vercel",
            "nvidia",
            "cerebras",
            "anthropic",
            "llmgateway",
            "openrouter",
        ])
    }

    /// Apply the legacy Vercel lower-case referer headers.
    pub fn apply_vercel(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "vercel" {
            return Ok(());
        }
        request
            .headers
            .entry("http-referer".to_string())
            .or_insert_with(|| "https://opencode.ai/".to_string());
        request
            .headers
            .entry("x-title".to_string())
            .or_insert_with(|| "opencode".to_string());
        Ok(())
    }

    /// Apply the NVIDIA tracking headers, preserving an explicit billing origin.
    pub fn apply_nvidia(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "nvidia" {
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
        request
            .headers
            .entry("X-BILLING-INVOKE-ORIGIN".to_string())
            .or_insert_with(|| "OpenCode".to_string());
        Ok(())
    }

    /// Apply the Cerebras third-party integration header.
    pub fn apply_cerebras(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "cerebras" {
            return Ok(());
        }
        request.headers.insert(
            "X-Cerebras-3rd-Party-Integration".to_string(),
            "opencode".to_string(),
        );
        Ok(())
    }

    /// Apply the Anthropic legacy beta headers.
    pub fn apply_anthropic(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "anthropic" {
            return Ok(());
        }
        request
            .headers
            .entry("anthropic-beta".to_string())
            .or_insert_with(|| {
                "interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14".to_string()
            });
        Ok(())
    }

    /// Apply the LLMGateway legacy referer headers when the integration is enabled.
    pub fn apply_llmgateway(
        provider_id: &str,
        enabled: bool,
        request: &mut ProviderRequest,
    ) -> CoreResult<()> {
        if provider_id != "llmgateway" || !enabled {
            return Ok(());
        }
        request.headers.insert(
            "HTTP-Referer".to_string(),
            "https://opencode.ai/".to_string(),
        );
        request
            .headers
            .insert("X-Title".to_string(), "opencode".to_string());
        request
            .headers
            .insert("X-Source".to_string(), "opencode".to_string());
        Ok(())
    }

    /// Apply the OpenRouter legacy referer headers.
    pub fn apply_openrouter(provider_id: &str, request: &mut ProviderRequest) -> CoreResult<()> {
        if provider_id != "openrouter" {
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
}
