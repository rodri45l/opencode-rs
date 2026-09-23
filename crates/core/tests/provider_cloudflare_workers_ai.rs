//! Port of packages/core/test/plugin/provider-cloudflare-workers-ai.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the plugin binds to `@ai-sdk/openai-compatible` only,
//! maps the Cloudflare account id to the endpoint URL unless a URL is already
//! configured, prefers the environment account id over a configured one,
//! expands `${CLOUDFLARE_ACCOUNT_ID}` in URLs, prefers the environment API key
//! over auth/config keys, and selects `languageModel` with the API model id.
//! Re-derived: the `AISDK`/`Catalog`/`PluginHost` service wiring and the SDK
//! factory are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_cloudflare_workers_ai::CloudflareWorkersAiPlugin;
use opencode_core::provider_sdk_plugins::LanguageSelector;

const NOTE: &str = "porting: cloudflare-workers-ai provider plugin not implemented";

fn headers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn binds_only_to_the_openai_compatible_package() {
    assert!(CloudflareWorkersAiPlugin::matches_package("@ai-sdk/openai-compatible").expect(NOTE));
    assert!(!CloudflareWorkersAiPlugin::matches_package("@ai-sdk/anthropic").expect(NOTE));
    assert!(!CloudflareWorkersAiPlugin::matches_package("test-provider").expect(NOTE));
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn maps_account_id_to_the_endpoint_url_unless_a_url_is_configured() {
    assert_eq!(
        CloudflareWorkersAiPlugin::resolve_endpoint(None, Some("acct"), None).expect(NOTE),
        Some("https://api.cloudflare.com/client/v4/accounts/acct/ai/v1".to_string())
    );
    assert_eq!(
        CloudflareWorkersAiPlugin::resolve_endpoint(
            Some("https://proxy.example/v1"),
            Some("acct"),
            None
        )
        .expect(NOTE),
        Some("https://proxy.example/v1".to_string())
    );
    assert_eq!(
        CloudflareWorkersAiPlugin::resolve_endpoint(None, None, None).expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn prefers_the_environment_account_id_over_the_configured_account_id() {
    assert_eq!(
        CloudflareWorkersAiPlugin::resolve_endpoint(
            None,
            Some("env-acct"),
            Some("configured-acct")
        )
        .expect(NOTE),
        Some("https://api.cloudflare.com/client/v4/accounts/env-acct/ai/v1".to_string())
    );
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn expands_account_id_variables_in_endpoint_urls() {
    assert_eq!(
        CloudflareWorkersAiPlugin::expand_account_id(
            "https://api.cloudflare.com/client/v4/accounts/${CLOUDFLARE_ACCOUNT_ID}/ai/v1",
            Some("acct"),
        )
        .expect(NOTE),
        "https://api.cloudflare.com/client/v4/accounts/acct/ai/v1"
    );
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn prefers_the_environment_api_key_and_keeps_custom_headers() {
    let authorization = CloudflareWorkersAiPlugin::authorization(
        Some("env-key"),
        Some("config-key"),
        Some("auth-key"),
    )
    .expect(NOTE);
    assert_eq!(authorization.as_deref(), Some("Bearer env-key"));

    let merged = CloudflareWorkersAiPlugin::merge_headers(
        authorization.as_deref(),
        &headers(&[("custom", "header")]),
    )
    .expect(NOTE);
    assert_eq!(
        merged.get("authorization").map(String::as_str),
        Some("Bearer env-key")
    );
    assert_eq!(merged.get("custom").map(String::as_str), Some("header"));
}

#[test]
#[ignore = "porting: cloudflare-workers-ai provider plugin not implemented"]
fn selects_language_model_with_the_api_model_id() {
    assert_eq!(
        CloudflareWorkersAiPlugin::select_language("@cf/api-model").expect(NOTE),
        LanguageSelector::LanguageModel
    );
}
