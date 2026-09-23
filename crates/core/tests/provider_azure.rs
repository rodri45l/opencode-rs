//! Port of packages/core/test/plugin/provider-azure.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Azure plugin binds to the exact `@ai-sdk/azure`
//! package, resolves `resourceName` from a configured non-blank value or
//! `AZURE_RESOURCE_NAME`, and selects language models through the legacy order
//! `responses` then `messages` then `chat` then `languageModel` (or `chat` when
//! completion URLs are requested), guarded to the `azure` provider. Re-derived:
//! the `AISDK`/`Catalog` service wiring, SDK factory and per-call/per-model
//! option plumbing are collapsed to the resolved decision.

use std::collections::BTreeMap;

use opencode_core::provider_azure::AzurePlugin;
use opencode_core::provider_sdk_plugins::{
    LanguageQuery, LanguageSelection, LanguageSelector, ProviderSdkPlugins, SdkCapabilities,
};

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

#[test]
fn resolves_resource_name_from_env() {
    assert_eq!(
        AzurePlugin::resolve_resource_name(None, &env(&[("AZURE_RESOURCE_NAME", "from-env")]))
            .unwrap(),
        Some("from-env".to_string())
    );
}

#[test]
fn keeps_an_explicit_resource_name_over_env() {
    assert_eq!(
        AzurePlugin::resolve_resource_name(
            Some("from-config"),
            &env(&[("AZURE_RESOURCE_NAME", "from-env")])
        )
        .unwrap(),
        Some("from-config".to_string())
    );
}

#[test]
fn falls_back_to_env_for_blank_or_whitespace_resource_names() {
    let values = env(&[("AZURE_RESOURCE_NAME", "from-env")]);
    assert_eq!(
        AzurePlugin::resolve_resource_name(Some(""), &values).unwrap(),
        Some("from-env".to_string())
    );
    assert_eq!(
        AzurePlugin::resolve_resource_name(Some("   "), &values).unwrap(),
        Some("from-env".to_string())
    );
}

#[test]
fn selects_chat_only_for_completion_urls() {
    let query = LanguageQuery {
        plugin: "azure",
        provider_id: "azure",
        model_id: "deployment",
        api_id: "deployment",
        capabilities: caps_all(),
        use_completion_urls: true,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Chat,
            model_id: "deployment".to_string(),
        })
    );
}

#[test]
fn uses_the_legacy_selector_order_and_provider_guard() {
    let query = LanguageQuery {
        plugin: "azure",
        provider_id: "azure",
        model_id: "deployment",
        api_id: "deployment",
        capabilities: caps_all(),
        use_completion_urls: false,
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Responses,
            model_id: "deployment".to_string(),
        })
    );

    let ignored = LanguageQuery {
        provider_id: "openai",
        ..query
    };
    assert_eq!(ProviderSdkPlugins::select_language(&ignored).unwrap(), None);
}

#[test]
fn falls_back_through_the_legacy_selector_order() {
    let messages = SdkCapabilities {
        messages: true,
        chat: true,
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&LanguageQuery {
            plugin: "azure",
            provider_id: "azure",
            model_id: "messages-deployment",
            api_id: "messages-deployment",
            capabilities: messages,
            use_completion_urls: false,
        })
        .unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Messages,
            model_id: "messages-deployment".to_string(),
        })
    );

    let language = SdkCapabilities {
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&LanguageQuery {
            plugin: "azure",
            provider_id: "azure",
            model_id: "language-deployment",
            api_id: "language-deployment",
            capabilities: language,
            use_completion_urls: false,
        })
        .unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::LanguageModel,
            model_id: "language-deployment".to_string(),
        })
    );
}
