//! Port of packages/core/test/plugin/provider-openrouter.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the OpenRouter plugin is registered, applies its legacy
//! referer headers only to the exact `openrouter` provider id, binds only to the
//! exact `@openrouter/ai-sdk-provider` package, and disables the
//! `openai/gpt-5-chat` alias for the `openrouter` provider only. Re-derived: the
//! `AISDK`/`Catalog` service wiring and SDK factory are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_header_plugins::ProviderHeaderPlugins;
use opencode_core::provider_plugins::ProviderRequest;
use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;
use serde_json::json;

fn request(headers: &[(&str, &str)]) -> ProviderRequest {
    ProviderRequest {
        headers: headers
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<BTreeMap<_, _>>(),
        body: json!({}),
    }
}

#[test]
fn is_registered_so_legacy_behavior_can_be_applied() {
    assert!(ProviderHeaderPlugins::registered_ids()
        .unwrap()
        .contains(&"openrouter"));
}

#[test]
fn applies_legacy_referer_headers_only_to_openrouter() {
    let mut openrouter = request(&[("Existing", "value")]);
    ProviderHeaderPlugins::apply_openrouter("openrouter", &mut openrouter).unwrap();
    assert_eq!(
        openrouter.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        openrouter.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        openrouter.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );

    let mut nvidia = request(&[]);
    ProviderHeaderPlugins::apply_openrouter("nvidia", &mut nvidia).unwrap();
    assert!(nvidia.headers.is_empty());
}

#[test]
fn binds_only_to_the_exact_openrouter_package() {
    assert!(
        ProviderSdkPlugins::matches_package("openrouter", "@openrouter/ai-sdk-provider").unwrap()
    );
    assert!(
        !ProviderSdkPlugins::matches_package("openrouter", "@ai-sdk/openai-compatible").unwrap()
    );
}

#[test]
fn disables_the_openrouter_gpt_5_chat_alias_without_affecting_others() {
    assert!(
        ProviderSdkPlugins::disables_model("openrouter", "openrouter", "openai/gpt-5-chat")
            .unwrap()
    );
    assert!(
        !ProviderSdkPlugins::disables_model("openrouter", "openrouter", "openai/gpt-5").unwrap()
    );
    assert!(
        !ProviderSdkPlugins::disables_model("openrouter", "openai", "openai/gpt-5-chat").unwrap()
    );
    assert!(!ProviderSdkPlugins::disables_model(
        "openrouter",
        "custom-openrouter",
        "gpt-5-chat-latest"
    )
    .unwrap());
}
