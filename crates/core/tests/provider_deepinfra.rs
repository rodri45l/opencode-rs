//! Port of packages/core/test/plugin/provider-deepinfra.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the DeepInfra plugin binds only to the exact
//! `@ai-sdk/deepinfra` package (rejecting prefixes and file URLs), names the SDK
//! provider `<provider>.chat`, and leaves language selection to the caller's
//! default `languageModel` fallback. Re-derived: the `AISDK` service wiring and
//! SDK factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

const NOTE: &str = "porting: deepinfra provider plugin not implemented";

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
#[ignore = "porting: deepinfra provider plugin not implemented"]
fn binds_only_to_the_exact_deepinfra_package() {
    assert!(ProviderSdkPlugins::matches_package("deepinfra", "@ai-sdk/deepinfra").expect(NOTE));
    assert!(!ProviderSdkPlugins::matches_package("deepinfra", "unmatched-package").expect(NOTE));
    assert!(
        !ProviderSdkPlugins::matches_package("deepinfra", "@ai-sdk/deepinfra-compatible")
            .expect(NOTE)
    );
    assert!(!ProviderSdkPlugins::matches_package(
        "deepinfra",
        "file:///tmp/@ai-sdk/deepinfra-provider.js"
    )
    .expect(NOTE));
}

#[test]
#[ignore = "porting: deepinfra provider plugin not implemented"]
fn names_the_sdk_provider_with_a_chat_suffix() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("deepinfra", "custom-deepinfra").expect(NOTE),
        "custom-deepinfra.chat"
    );
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("deepinfra", "deepinfra").expect(NOTE),
        "deepinfra.chat"
    );
}

#[test]
#[ignore = "porting: deepinfra provider plugin not implemented"]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "deepinfra",
        provider_id: "deepinfra",
        model_id: "meta-llama/Llama-3.3-70B-Instruct",
        api_id: "meta-llama/Llama-3.3-70B-Instruct",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        None
    );
}
