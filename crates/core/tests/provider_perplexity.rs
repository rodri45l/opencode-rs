//! Port of packages/core/test/plugin/provider-perplexity.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Perplexity plugin binds only to the exact
//! `@ai-sdk/perplexity` package, names the SDK provider `perplexity` regardless
//! of the model provider id, and leaves language selection to the default
//! `languageModel` fallback. Re-derived: the `AISDK` service wiring and SDK
//! factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

const NOTE: &str = "porting: perplexity provider plugin not implemented";

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
#[ignore = "porting: perplexity provider plugin not implemented"]
fn binds_only_to_the_exact_perplexity_package() {
    assert!(ProviderSdkPlugins::matches_package("perplexity", "@ai-sdk/perplexity").expect(NOTE));
    assert!(
        !ProviderSdkPlugins::matches_package("perplexity", "@ai-sdk/perplexity-compatible")
            .expect(NOTE)
    );
}

#[test]
#[ignore = "porting: perplexity provider plugin not implemented"]
fn uses_the_canonical_perplexity_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("perplexity", "perplexity").expect(NOTE),
        "perplexity"
    );
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("perplexity", "custom-perplexity").expect(NOTE),
        "perplexity"
    );
}

#[test]
#[ignore = "porting: perplexity provider plugin not implemented"]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "perplexity",
        provider_id: "perplexity",
        model_id: "alias",
        api_id: "sonar",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        None
    );
}
