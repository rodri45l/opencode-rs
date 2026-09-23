//! Dynamic provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/dynamic.ts`: the plugin builds an SDK from
//! a provider factory export, never overrides an SDK supplied by an earlier
//! plugin, injects the provider id as the SDK factory name, uses the model's API
//! id for the default language model, and loads npm packages through their
//! resolved import entrypoint when one is available. The `AISDK`/`PluginHost`
//! wiring, dynamic `import`, and the module loader are dropped.

use crate::CoreResult;

/// The dynamic provider plugin.
#[derive(Debug, Default)]
pub struct DynamicProviderPlugin;

impl DynamicProviderPlugin {
    /// Whether an SDK from an earlier plugin may be replaced. An already
    /// supplied SDK is never overridden.
    pub fn should_override_sdk(existing_sdk: bool) -> CoreResult<bool> {
        Ok(!existing_sdk)
    }

    /// The SDK factory name injected for a provider id.
    pub fn sdk_name(provider_id: &str) -> CoreResult<String> {
        Ok(provider_id.to_string())
    }

    /// The default language model id: the model's API id.
    pub fn default_language_model_id(api_id: &str) -> CoreResult<String> {
        Ok(api_id.to_string())
    }

    /// The module source to import: a resolved npm entrypoint when available,
    /// otherwise the package specifier.
    pub fn import_source(package: &str, npm_entrypoint: Option<&str>) -> CoreResult<String> {
        Ok(npm_entrypoint
            .filter(|value| !value.is_empty())
            .unwrap_or(package)
            .to_string())
    }
}
