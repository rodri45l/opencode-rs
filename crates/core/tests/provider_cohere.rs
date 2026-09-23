//! Port of packages/core/test/plugin/provider-cohere.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Cohere plugin binds only to the exact `@ai-sdk/cohere`
//! package (not `@ai-sdk/openai-compatible`), names the SDK provider
//! `<provider>.chat`, and leaves language selection to the default
//! `languageModel` fallback. Re-derived: the `AISDK` service wiring and SDK
//! factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

const NOTE: &str = "porting: cohere provider plugin not implemented";

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
#[ignore = "porting: cohere provider plugin not implemented"]
fn binds_only_to_the_exact_cohere_package() {
    assert!(ProviderSdkPlugins::matches_package("cohere", "@ai-sdk/cohere").expect(NOTE));
    assert!(
        !ProviderSdkPlugins::matches_package("cohere", "@ai-sdk/openai-compatible").expect(NOTE)
    );
}

#[test]
#[ignore = "porting: cohere provider plugin not implemented"]
fn uses_the_model_provider_id_as_the_bundled_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("cohere", "custom-cohere").expect(NOTE),
        "custom-cohere.chat"
    );
}

#[test]
#[ignore = "porting: cohere provider plugin not implemented"]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "cohere",
        provider_id: "cohere",
        model_id: "alias",
        api_id: "command-r-plus",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        None
    );
}
