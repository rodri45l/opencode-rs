//! Port of packages/core/test/plugin/provider-vercel.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Vercel plugin is registered, applies the legacy
//! lower-case referer headers (`http-referer`, `x-title`) only to the exact
//! `vercel` provider id, never the upper-case names, merges with existing
//! headers, binds only to the exact `@ai-sdk/vercel` package, and names the SDK
//! provider `<provider>.chat`. Re-derived: the `AISDK`/`Catalog` service wiring
//! and SDK factory are dropped.

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
fn is_registered_so_legacy_referer_headers_can_be_applied() {
    assert!(ProviderHeaderPlugins::registered_ids()
        .unwrap()
        .contains(&"vercel"));
}

#[test]
fn applies_the_legacy_lower_case_referer_headers_only_to_vercel() {
    let mut vercel = request(&[("Existing", "value")]);
    ProviderHeaderPlugins::apply_vercel("vercel", &mut vercel).unwrap();

    assert_eq!(
        vercel.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        vercel.headers.get("http-referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        vercel.headers.get("x-title").map(String::as_str),
        Some("opencode")
    );

    let mut gateway = request(&[]);
    ProviderHeaderPlugins::apply_vercel("gateway", &mut gateway).unwrap();
    assert!(gateway.headers.is_empty());
}

#[test]
fn does_not_add_the_legacy_upper_case_referer_headers() {
    let mut vercel = request(&[]);
    ProviderHeaderPlugins::apply_vercel("vercel", &mut vercel).unwrap();

    assert!(!vercel.headers.contains_key("HTTP-Referer"));
    assert!(!vercel.headers.contains_key("X-Title"));
}

#[test]
fn binds_to_the_vercel_sdk_package() {
    assert!(ProviderSdkPlugins::matches_package("vercel", "@ai-sdk/vercel").unwrap());
}

#[test]
fn names_the_sdk_provider_with_a_chat_suffix() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("vercel", "custom-vercel").unwrap(),
        "vercel.chat"
    );
}
