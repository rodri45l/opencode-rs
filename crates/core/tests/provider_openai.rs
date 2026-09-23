//! Port of packages/core/test/plugin/provider-openai.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the OpenAI plugin binds only to the exact `@ai-sdk/openai`
//! package, selects language models through the `responses` accessor using the
//! model API id, ignores other providers/packages, and disables the
//! `gpt-5-chat-latest` model for the exact `openai` provider only. Re-derived:
//! the `AISDK`/`Integration` service wiring, OAuth methods and SDK factory are
//! dropped.

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
fn binds_only_to_the_exact_openai_package() {
    assert!(ProviderSdkPlugins::matches_package("openai", "@ai-sdk/openai").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("openai", "@ai-sdk/openai-compatible").unwrap());
}

#[test]
fn names_the_sdk_provider_with_the_model_provider_id() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("openai", "custom-openai").unwrap(),
        "custom-openai"
    );
}

#[test]
fn selects_responses_with_the_model_api_id() {
    let query = LanguageQuery {
        plugin: "openai",
        provider_id: "openai",
        model_id: "alias",
        api_id: "gpt-5",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Responses,
            model_id: "gpt-5".to_string(),
        })
    );
}

#[test]
fn ignores_non_openai_providers() {
    let query = LanguageQuery {
        plugin: "openai",
        provider_id: "anthropic",
        model_id: "gpt-5",
        api_id: "gpt-5",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(ProviderSdkPlugins::select_language(&query).unwrap(), None);
}

#[test]
fn disables_gpt_5_chat_latest_for_the_exact_openai_provider() {
    assert!(ProviderSdkPlugins::disables_model("openai", "openai", "gpt-5-chat-latest").unwrap());
    assert!(!ProviderSdkPlugins::disables_model("openai", "openai", "gpt-5").unwrap());
    assert!(
        !ProviderSdkPlugins::disables_model("openai", "custom-openai", "gpt-5-chat-latest")
            .unwrap()
    );
}
