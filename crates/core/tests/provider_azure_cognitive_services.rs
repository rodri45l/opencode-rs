//! Port of packages/core/test/plugin/provider-azure-cognitive-services.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Azure Cognitive Services plugin derives the API URL
//! `https://{resource}.cognitiveservices.azure.com/openai` from
//! `AZURE_COGNITIVE_SERVICES_RESOURCE_NAME` (leaving it unset otherwise), and
//! selects language models through the legacy order `responses` then `messages`
//! then `chat` then `languageModel` (or `chat` when completion URLs are
//! requested), guarded to the Azure provider. Re-derived: the `Catalog`/`AISDK`
//! service wiring and SDK factory are collapsed to the resolved decisions.

use std::collections::BTreeMap;

use opencode_core::provider_azure_cognitive_services::AzureCognitiveServicesPlugin;
use opencode_core::provider_sdk_plugins::{
    LanguageQuery, LanguageSelection, LanguageSelector, ProviderSdkPlugins, SdkCapabilities,
};

const NOTE: &str = "porting: azure-cognitive-services provider plugin not implemented";

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
#[ignore = "porting: azure-cognitive-services provider plugin not implemented"]
fn maps_the_resource_env_var_to_the_base_url() {
    assert_eq!(
        AzureCognitiveServicesPlugin::base_url(&env(&[(
            "AZURE_COGNITIVE_SERVICES_RESOURCE_NAME",
            "cognitive"
        )]))
        .expect(NOTE),
        Some("https://cognitive.cognitiveservices.azure.com/openai".to_string())
    );
}

#[test]
#[ignore = "porting: azure-cognitive-services provider plugin not implemented"]
fn leaves_the_base_url_unset_without_the_resource_env() {
    assert_eq!(
        AzureCognitiveServicesPlugin::base_url(&env(&[])).expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: azure-cognitive-services provider plugin not implemented"]
fn selects_chat_only_for_completion_urls() {
    let query = LanguageQuery {
        plugin: "azure-cognitive-services",
        provider_id: "azure-cognitive-services",
        model_id: "deployment",
        api_id: "deployment",
        capabilities: SdkCapabilities {
            responses: true,
            messages: true,
            chat: true,
            language_model: true,
        },
        use_completion_urls: true,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        Some(LanguageSelection {
            selector: LanguageSelector::Chat,
            model_id: "deployment".to_string(),
        })
    );
}

#[test]
#[ignore = "porting: azure-cognitive-services provider plugin not implemented"]
fn uses_the_legacy_selector_order_and_provider_guard() {
    let query = LanguageQuery {
        plugin: "azure-cognitive-services",
        provider_id: "azure-cognitive-services",
        model_id: "deployment",
        api_id: "deployment",
        capabilities: SdkCapabilities {
            responses: true,
            messages: true,
            chat: true,
            language_model: true,
        },
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).expect(NOTE),
        Some(LanguageSelection {
            selector: LanguageSelector::Responses,
            model_id: "deployment".to_string(),
        })
    );

    let ignored = LanguageQuery {
        provider_id: "openai",
        ..query
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&ignored).expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: azure-cognitive-services provider plugin not implemented"]
fn falls_back_from_responses_to_messages_chat_then_language_model() {
    let messages = SdkCapabilities {
        messages: true,
        chat: true,
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&LanguageQuery {
            plugin: "azure-cognitive-services",
            provider_id: "azure-cognitive-services",
            model_id: "messages-deployment",
            api_id: "messages-deployment",
            capabilities: messages,
            use_completion_urls: false,
        })
        .expect(NOTE),
        Some(LanguageSelection {
            selector: LanguageSelector::Messages,
            model_id: "messages-deployment".to_string(),
        })
    );

    let chat = SdkCapabilities {
        chat: true,
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&LanguageQuery {
            plugin: "azure-cognitive-services",
            provider_id: "azure-cognitive-services",
            model_id: "chat-deployment",
            api_id: "chat-deployment",
            capabilities: chat,
            use_completion_urls: false,
        })
        .expect(NOTE),
        Some(LanguageSelection {
            selector: LanguageSelector::Chat,
            model_id: "chat-deployment".to_string(),
        })
    );

    let language = SdkCapabilities {
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&LanguageQuery {
            plugin: "azure-cognitive-services",
            provider_id: "azure-cognitive-services",
            model_id: "language-deployment",
            api_id: "language-deployment",
            capabilities: language,
            use_completion_urls: false,
        })
        .expect(NOTE),
        Some(LanguageSelection {
            selector: LanguageSelector::LanguageModel,
            model_id: "language-deployment".to_string(),
        })
    );
}
