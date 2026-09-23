//! Built-in SDK provider plugins (re-derived behavioural subset).
//!
//! Ports the observable behaviour of the `AISDK`-bound provider plugins in
//! `packages/core/src/plugin/provider/*`: each plugin binds to an exact SDK
//! package, derives the SDK provider name from the provider id, selects a
//! language-model accessor, defaults `includeUsage` for the openai-compatible
//! fallback, and may disable specific catalog models. The Effect service wiring
//! and the SDK factories themselves are dropped; the pure decisions remain.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Which SDK accessor a provider plugin uses to select a language model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageSelector {
    /// `sdk.languageModel(model_id)`.
    LanguageModel,
    /// `sdk.responses(model_id)`.
    Responses,
    /// `sdk.messages(model_id)`.
    Messages,
    /// `sdk.chat(model_id)`.
    Chat,
}

/// The accessors an SDK advertises, used to resolve fallback order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SdkCapabilities {
    /// The SDK exposes `responses`.
    pub responses: bool,
    /// The SDK exposes `messages`.
    pub messages: bool,
    /// The SDK exposes `chat`.
    pub chat: bool,
    /// The SDK exposes `languageModel`.
    pub language_model: bool,
}

/// A resolved language-model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSelection {
    /// The accessor the plugin chose.
    pub selector: LanguageSelector,
    /// The model id passed to the accessor.
    pub model_id: String,
}

/// A built-in SDK provider plugin descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdkPlugin {
    /// Plugin id (for example `"anthropic"`).
    pub id: &'static str,
    /// The exact npm package this plugin binds to.
    pub package: &'static str,
}

/// Inputs for resolving a plugin's language-model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageQuery<'a> {
    /// Plugin id.
    pub plugin: &'a str,
    /// Provider id the model belongs to.
    pub provider_id: &'a str,
    /// Catalog model id (the alias the user selected).
    pub model_id: &'a str,
    /// Wire API model id advertised by the model.
    pub api_id: &'a str,
    /// Accessors the SDK advertises.
    pub capabilities: SdkCapabilities,
    /// Whether the provider is configured to use legacy completion URLs.
    pub use_completion_urls: bool,
}

/// Registry of built-in SDK provider plugins.
#[derive(Debug, Default)]
pub struct ProviderSdkPlugins;

impl ProviderSdkPlugins {
    /// Find the plugin bound to the exact `package`, preserving registry order.
    pub fn by_package(_package: &str) -> CoreResult<Option<SdkPlugin>> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::by_package",
        ))
    }

    /// Whether `plugin` handles `package` under that plugin's matching rule
    /// (most are exact; the openai-compatible fallback also matches package
    /// paths that contain it).
    pub fn matches_package(_plugin: &str, _package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::matches_package",
        ))
    }

    /// The SDK provider name a plugin reports for `provider_id`.
    pub fn sdk_provider_name(_plugin: &str, _provider_id: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::sdk_provider_name",
        ))
    }

    /// Resolve the language-model selection for a provider turn.
    pub fn select_language(_query: &LanguageQuery<'_>) -> CoreResult<Option<LanguageSelection>> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::select_language",
        ))
    }

    /// The defaulted `includeUsage` option for an OpenAI-compatible SDK.
    pub fn include_usage(_options: &Value) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::include_usage",
        ))
    }

    /// Whether `plugin` disables `model_id` for `provider_id` in catalog transforms.
    pub fn disables_model(_plugin: &str, _provider_id: &str, _model_id: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "provider_sdk_plugins::ProviderSdkPlugins::disables_model",
        ))
    }
}
