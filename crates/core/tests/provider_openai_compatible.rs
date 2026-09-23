//! Port of packages/core/test/plugin/provider-openai-compatible.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the OpenAI-compatible fallback defaults `includeUsage` to
//! `true` while preserving an explicit `false`, names the SDK provider
//! `<provider>.chat`, and binds only to the exact `@ai-sdk/openai-compatible`
//! package. Re-derived: the `AISDK` hook/sentinel wiring and SDK factory are
//! dropped.

use opencode_core::provider_sdk_plugins::ProviderSdkPlugins;
use serde_json::json;

const NOTE: &str = "porting: openai-compatible provider plugin not implemented";

#[test]
#[ignore = "porting: openai-compatible provider plugin not implemented"]
fn preserves_explicit_include_usage_false_and_defaults_to_true() {
    assert!(ProviderSdkPlugins::include_usage(&json!({})).expect(NOTE));
    assert!(!ProviderSdkPlugins::include_usage(&json!({ "includeUsage": false })).expect(NOTE));
    assert!(ProviderSdkPlugins::include_usage(&json!({ "includeUsage": true })).expect(NOTE));
}

#[test]
#[ignore = "porting: openai-compatible provider plugin not implemented"]
fn uses_the_provider_id_as_the_openai_compatible_provider_name() {
    assert_eq!(
        ProviderSdkPlugins::sdk_provider_name("openai-compatible", "custom-provider").expect(NOTE),
        "custom-provider.chat"
    );
}

#[test]
#[ignore = "porting: openai-compatible provider plugin not implemented"]
fn binds_to_the_openai_compatible_package() {
    assert!(
        ProviderSdkPlugins::matches_package("openai-compatible", "@ai-sdk/openai-compatible")
            .expect(NOTE)
    );
}
