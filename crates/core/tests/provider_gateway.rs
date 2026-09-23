//! Port of packages/core/test/plugin/provider-gateway.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Gateway plugin binds only to the exact
//! `@ai-sdk/gateway` package (not `@ai-sdk/vercel`) and passes the model
//! provider id as the SDK name. Re-derived: the `AISDK` service wiring and SDK
//! factory are dropped.

use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;

const NOTE: &str = "porting: gateway provider plugin not implemented";

#[test]
#[ignore = "porting: gateway provider plugin not implemented"]
fn binds_only_to_the_exact_gateway_package() {
    assert!(ProviderSdkPlugins::matches_package("gateway", "@ai-sdk/gateway").expect(NOTE));
    assert!(!ProviderSdkPlugins::matches_package("gateway", "@ai-sdk/vercel").expect(NOTE));
}

#[test]
#[ignore = "porting: gateway provider plugin not implemented"]
fn passes_the_model_provider_id_as_the_sdk_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("gateway", "vercel").expect(NOTE),
        "vercel"
    );
}
