//! Port of packages/core/test/plugin/provider-gateway.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Gateway plugin binds only to the exact
//! `@ai-sdk/gateway` package (not `@ai-sdk/vercel`) and passes the model
//! provider id as the SDK name. Re-derived: the `AISDK` service wiring and SDK
//! factory are dropped.

use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;

#[test]
fn binds_only_to_the_exact_gateway_package() {
    assert!(ProviderSdkPlugins::matches_package("gateway", "@ai-sdk/gateway").unwrap());
    assert!(!ProviderSdkPlugins::matches_package("gateway", "@ai-sdk/vercel").unwrap());
}

#[test]
fn passes_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("gateway", "vercel").unwrap(),
        "vercel"
    );
}
