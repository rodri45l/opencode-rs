//! Port of packages/core/test/plugin/provider-xai.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the xAI plugin binds only to the exact `@ai-sdk/xai`
//! package, selects language models through the `responses` accessor using the
//! model API id, and ignores other providers. Re-derived: the `AISDK` service
//! wiring and SDK factory are dropped.

use opencode_core::provider_sdk_plugins::{
    LanguageQuery, LanguageSelection, LanguageSelector, ProviderSdkPlugins, SdkCapabilities,
};

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
fn binds_only_to_the_exact_xai_package() {
    assert!(ProviderSdkPlugins::matches_package("xai", "@ai-sdk/xai").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("xai", "@ai-sdk/openai-compatible").unwrap());
}

#[test]
fn uses_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("xai", "custom-xai").unwrap(),
        "custom-xai"
    );
}

#[test]
fn selects_responses_with_the_model_api_id() {
    let query = LanguageQuery {
        plugin: "xai",
        provider_id: "xai",
        model_id: "alias",
        api_id: "grok-4",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Responses,
            model_id: "grok-4".to_string(),
        })
    );
}

#[test]
fn ignores_non_xai_providers() {
    let query = LanguageQuery {
        plugin: "xai",
        provider_id: "openai",
        model_id: "grok-4",
        api_id: "grok-4",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(ProviderSdkPlugins::select_language(&query).unwrap(), None);
}
