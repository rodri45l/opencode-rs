//! Legacy provider header plugins (re-derived behavioural subset).
//!
//! Ports the observable behaviour of the legacy referer/tracking-header
//! providers in `packages/core/src/plugin/provider/*`: each applies a fixed set
//! of request headers to exactly its own provider id, merges with configured
//! headers, and (for llmgateway) is guarded by the enabled flag. The
//! `Catalog`/`PluginHost` wiring is replaced by a pure request transform.

use crate::provider_plugins::ProviderRequest;
use crate::{CoreError, CoreResult};

/// The legacy provider header plugins.
#[derive(Debug, Default)]
pub struct ProviderHeaderPlugins;

impl ProviderHeaderPlugins {
    /// Registered legacy header plugin ids in precedence order.
    pub fn registered_ids() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::registered_ids",
        ))
    }

    /// Apply the legacy Vercel lower-case referer headers.
    pub fn apply_vercel(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_vercel",
        ))
    }

    /// Apply the NVIDIA tracking headers, preserving an explicit billing origin.
    pub fn apply_nvidia(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_nvidia",
        ))
    }

    /// Apply the Cerebras third-party integration header.
    pub fn apply_cerebras(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_cerebras",
        ))
    }

    /// Apply the Anthropic legacy beta headers.
    pub fn apply_anthropic(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_anthropic",
        ))
    }

    /// Apply the LLMGateway legacy referer headers when the integration is enabled.
    pub fn apply_llmgateway(
        _provider_id: &str,
        _enabled: bool,
        _request: &mut ProviderRequest,
    ) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_llmgateway",
        ))
    }

    /// Apply the OpenRouter legacy referer headers.
    pub fn apply_openrouter(_provider_id: &str, _request: &mut ProviderRequest) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "provider_header_plugins::ProviderHeaderPlugins::apply_openrouter",
        ))
    }
}
