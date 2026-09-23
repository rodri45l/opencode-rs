//! Dynamic provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/dynamic.ts`: the plugin builds an SDK from
//! a provider factory export, never overrides an SDK supplied by an earlier
//! plugin, injects the provider id as the SDK factory name, uses the model's API
//! id for the default language model, and loads npm packages through their
//! resolved import entrypoint when one is available. The `AISDK`/`PluginHost`
//! wiring, dynamic `import`, and the module loader are dropped.

use crate::{CoreError, CoreResult};

/// The dynamic provider plugin.
#[derive(Debug, Default)]
pub struct DynamicProviderPlugin;

impl DynamicProviderPlugin {
    /// Whether an SDK from an earlier plugin may be replaced. An already
    /// supplied SDK is never overridden.
    pub fn should_override_sdk(_existing_sdk: bool) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_dynamic::DynamicProviderPlugin::should_override_sdk",
        ))
    }

    /// The SDK factory name injected for a provider id.
    pub fn sdk_name(_provider_id: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_dynamic::DynamicProviderPlugin::sdk_name",
        ))
    }

    /// The default language model id: the model's API id.
    pub fn default_language_model_id(_api_id: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_dynamic::DynamicProviderPlugin::default_language_model_id",
        ))
    }

    /// The module source to import: a resolved npm entrypoint when available,
    /// otherwise the package specifier.
    pub fn import_source(_package: &str, _npm_entrypoint: Option<&str>) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_dynamic::DynamicProviderPlugin::import_source",
        ))
    }
}
