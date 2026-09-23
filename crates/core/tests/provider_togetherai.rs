//! Port of packages/core/test/plugin/provider-togetherai.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the TogetherAI plugin binds only to the exact
//! `@ai-sdk/togetherai` package (rejecting file URLs), names the SDK provider
//! `togetherai.chat` regardless of the model provider id, and leaves language
//! selection to the default `languageModel` accessor. Re-derived: the `AISDK`
//! service wiring and SDK factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

const NOTE: &str = "porting: togetherai provider plugin not implemented";

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
#[ignore = "porting: togetherai provider plugin not implemented"]
fn binds_only_to_the_exact_togetherai_package() {
    assert!(ProviderSdkPlugins::matches_package("togetherai", "@ai-sdk/togetherai").expect(NOTE));
    assert!(!ProviderSdkPlugins::matches_package(
        "togetherai",
        "file:///tmp/@ai-sdk/togetherai-provider.js"
    )
    .expect(NOTE));
}

#[test]
#[ignore = "porting: togetherai provider plugin not implemented"]
fn uses_the_canonical_togetherai_sdk_name_for_custom_providers() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("togetherai", "custom-togetherai").expect(NOTE),
        "togetherai.chat"
    );
}

#[test]
#[ignore = "porting: togetherai provider plugin not implemented"]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "togetherai",
        provider_id: "togetherai",
        model_id: "alias",
        api_id: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        None
    );
}
