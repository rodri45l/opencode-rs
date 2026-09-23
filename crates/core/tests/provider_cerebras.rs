//! Port of packages/core/test/plugin/provider-cerebras.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Cerebras plugin applies the legacy
//! `X-Cerebras-3rd-Party-Integration: opencode` header only to the exact
//! `cerebras` provider id, merges with existing headers, binds only to
//! `@ai-sdk/cerebras`, and uses the model provider id as the SDK name.
//! Re-derived: the `AISDK`/`Catalog` service wiring and SDK factory are dropped.

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
fn applies_the_legacy_integration_header_only_to_cerebras() {
    let mut cerebras = request(&[("Existing", "1")]);
    ProviderHeaderPlugins::apply_cerebras("cerebras", &mut cerebras).unwrap();
    assert_eq!(
        cerebras
            .headers
            .get("X-Cerebras-3rd-Party-Integration")
            .map(String::as_str),
        Some("opencode")
    );
    assert_eq!(
        cerebras.headers.get("Existing").map(String::as_str),
        Some("1")
    );

    let mut groq = request(&[]);
    ProviderHeaderPlugins::apply_cerebras("groq", &mut groq).unwrap();
    assert!(groq.headers.is_empty());
}

#[test]
fn binds_only_to_the_exact_cerebras_package() {
    assert!(ProviderSdkPlugins::matches_package("cerebras", "@ai-sdk/cerebras").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("cerebras", "@ai-sdk/groq").unwrap());
}

#[test]
fn uses_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("cerebras", "custom-cerebras").unwrap(),
        "custom-cerebras"
    );
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("cerebras", "cerebras").unwrap(),
        "cerebras"
    );
}
