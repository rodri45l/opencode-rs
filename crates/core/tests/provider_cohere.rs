//! Port of packages/core/test/plugin/provider-cohere.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Cohere plugin binds only to the exact `@ai-sdk/cohere`
//! package (not `@ai-sdk/openai-compatible`), names the SDK provider
//! `<provider>.chat`, and leaves language selection to the default
//! `languageModel` fallback. Re-derived: the `AISDK` service wiring and SDK
//! factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
fn binds_only_to_the_exact_cohere_package() {
    assert!(ProviderSdkPlugins::matches_package("cohere", "@ai-sdk/cohere").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("cohere", "@ai-sdk/openai-compatible").unwrap());
}

#[test]
fn uses_the_model_provider_id_as_the_bundled_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("cohere", "custom-cohere").unwrap(),
        "custom-cohere.chat"
    );
}

#[test]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "cohere",
        provider_id: "cohere",
        model_id: "alias",
        api_id: "command-r-plus",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(ProviderSdkPlugins::select_language(&query).unwrap(), None);
}
