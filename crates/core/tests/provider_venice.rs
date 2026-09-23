//! Port of packages/core/test/plugin/provider-venice.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Venice plugin binds only to the exact
//! `venice-ai-sdk-provider` package (rejecting file URLs and other packages),
//! names the SDK provider `<provider>.chat`, and leaves language selection to
//! the default `languageModel` fallback. Re-derived: the `AISDK` service wiring
//! and SDK factory are dropped.

use opencode_core::provider_sdk_plugins::{LanguageQuery, ProviderSdkPlugins, SdkCapabilities};

const NOTE: &str = "porting: venice provider plugin not implemented";

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
#[ignore = "porting: venice provider plugin not implemented"]
fn binds_only_to_the_bundled_venice_package() {
    assert!(ProviderSdkPlugins::matches_package("venice", "venice-ai-sdk-provider").expect(NOTE));
    assert!(!ProviderSdkPlugins::matches_package(
        "venice",
        "file:///tmp/venice-ai-sdk-provider.js"
    )
    .expect(NOTE));
    assert!(
        !ProviderSdkPlugins::matches_package("venice", "@ai-sdk/openai-compatible").expect(NOTE)
    );
}

#[test]
#[ignore = "porting: venice provider plugin not implemented"]
fn uses_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("venice", "custom-venice").expect(NOTE),
        "custom-venice.chat"
    );
}

#[test]
#[ignore = "porting: venice provider plugin not implemented"]
fn leaves_language_selection_to_the_default_fallback() {
    let query = LanguageQuery {
        plugin: "venice",
        provider_id: "venice",
        model_id: "alias",
        api_id: "alias",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        None
    );
}
