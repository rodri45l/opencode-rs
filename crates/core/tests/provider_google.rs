//! Port of packages/core/test/plugin/provider-google.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Google plugin binds only to the exact `@ai-sdk/google`
//! package (not `@ai-sdk/google-vertex`), names the SDK provider with the model
//! provider id, and leaves language selection on the default `languageModel`
//! accessor. Re-derived: the `AISDK` service wiring and SDK factory are dropped.

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
fn binds_only_to_the_exact_google_package() {
    assert!(ProviderSdkPlugins::matches_package("google", "@ai-sdk/google").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("google", "@ai-sdk/google-vertex").unwrap());
}

#[test]
fn uses_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("google", "custom-google").unwrap(),
        "custom-google"
    );
}

#[test]
fn leaves_language_selection_to_the_default_language_model() {
    let query = LanguageQuery {
        plugin: "google",
        provider_id: "custom-google",
        model_id: "alias",
        api_id: "gemini-api",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(ProviderSdkPlugins::select_language(&query).unwrap(), None);
}
