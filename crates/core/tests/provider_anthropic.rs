//! Port of packages/core/test/plugin/provider-anthropic.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Anthropic plugin applies its legacy beta headers to
//! exactly the `anthropic` provider id while preserving configured headers,
//! ignores other providers, binds only to `@ai-sdk/anthropic`, and uses the
//! provider id as the SDK name. Re-derived: the `AISDK`/`Catalog` service wiring
//! and SDK factory are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_header_plugins::ProviderHeaderPlugins;
use opencode_core::provider_plugins::ProviderRequest;
use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;
use serde_json::json;

const NOTE: &str = "porting: anthropic provider plugin not implemented";

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
#[ignore = "porting: anthropic provider plugin not implemented"]
fn applies_the_legacy_beta_headers_and_preserves_existing_headers() {
    let mut anthropic = request(&[("Existing", "1")]);
    ProviderHeaderPlugins::apply_anthropic("anthropic", &mut anthropic).expect(NOTE);

    assert_eq!(
        anthropic.headers.get("anthropic-beta").map(String::as_str),
        Some("interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14")
    );
    assert_eq!(
        anthropic.headers.get("Existing").map(String::as_str),
        Some("1")
    );
}

#[test]
#[ignore = "porting: anthropic provider plugin not implemented"]
fn ignores_non_anthropic_providers() {
    let mut openai = request(&[]);
    ProviderHeaderPlugins::apply_anthropic("openai", &mut openai).expect(NOTE);

    assert!(!openai.headers.contains_key("anthropic-beta"));
}

#[test]
#[ignore = "porting: anthropic provider plugin not implemented"]
fn binds_to_the_anthropic_sdk_package() {
    assert!(ProviderSdkPlugins::matches_package("anthropic", "@ai-sdk/anthropic").expect(NOTE));
}

#[test]
#[ignore = "porting: anthropic provider plugin not implemented"]
fn uses_the_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("anthropic", "custom-anthropic").expect(NOTE),
        "custom-anthropic"
    );
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("anthropic", "anthropic").expect(NOTE),
        "anthropic"
    );
}
