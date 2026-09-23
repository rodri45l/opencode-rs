//! Port of packages/core/test/plugin/provider-alibaba.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Alibaba plugin binds only to the exact
//! `@ai-sdk/alibaba` package (not `@ai-sdk/openai-compatible`) and names the SDK
//! provider `alibaba.chat`. Re-derived: the `AISDK` service wiring, SDK factory
//! and the model-aliasing comparisons are dropped.

use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;

#[test]
fn binds_only_to_the_exact_alibaba_package() {
    assert!(ProviderSdkPlugins::matches_package("alibaba", "@ai-sdk/alibaba").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("alibaba", "@ai-sdk/openai-compatible").unwrap());
}

#[test]
fn uses_the_canonical_alibaba_chat_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("alibaba", "alibaba").unwrap(),
        "alibaba.chat"
    );
}
