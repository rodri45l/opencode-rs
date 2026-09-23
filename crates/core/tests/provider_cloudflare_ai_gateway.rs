//! Port of packages/core/test/plugin/provider-cloudflare-ai-gateway.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the plugin binds to `ai-gateway-provider` only, requires
//! account, gateway, and token before creating the SDK, prefers Cloudflare
//! environment values over auth/config-derived options, accepts a `gatewayId`
//! option as the gateway id, falls back to `CF_AIG_TOKEN` when
//! `CLOUDFLARE_API_TOKEN` is unset, and resolves legacy metadata from the
//! `cf-aig-metadata` JSON header when no `metadata` option is present.
//! Re-derived: the `AISDK`/`Catalog`/`PluginHost` service wiring and the SDK
//! factory are dropped.

use opencode_core::provider_cloudflare_ai_gateway::{
    CloudflareAiGatewayPlugin, CloudflareGatewayConfig, CloudflareGatewayEnv,
    CloudflareGatewayOptions,
};
use serde_json::json;

const NOTE: &str = "porting: cloudflare-ai-gateway provider plugin not implemented";

fn env(
    account_id: Option<&str>,
    gateway: Option<&str>,
    api_token: Option<&str>,
    cf_aig_token: Option<&str>,
) -> CloudflareGatewayEnv {
    CloudflareGatewayEnv {
        account_id: account_id.map(str::to_string),
        gateway: gateway.map(str::to_string),
        api_token: api_token.map(str::to_string),
        cf_aig_token: cf_aig_token.map(str::to_string),
    }
}

fn config(account_id: &str, gateway: &str, api_key: &str) -> CloudflareGatewayConfig {
    CloudflareGatewayConfig {
        account_id: account_id.to_string(),
        gateway: gateway.to_string(),
        api_key: api_key.to_string(),
    }
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn binds_only_to_the_ai_gateway_package() {
    assert!(CloudflareAiGatewayPlugin::matches_package("ai-gateway-provider").expect(NOTE));
    assert!(!CloudflareAiGatewayPlugin::matches_package("@ai-sdk/openai-compatible").expect(NOTE));
    assert!(!CloudflareAiGatewayPlugin::matches_package("test-provider").expect(NOTE));
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn requires_account_gateway_and_token() {
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(Some("acct"), Some("gateway"), Some("token"), None),
            &CloudflareGatewayOptions::default(),
        )
        .expect(NOTE),
        Some(config("acct", "gateway", "token"))
    );
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(None, Some("gateway"), Some("token"), None),
            &CloudflareGatewayOptions::default(),
        )
        .expect(NOTE),
        None
    );
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(Some("acct"), None, Some("token"), None),
            &CloudflareGatewayOptions::default(),
        )
        .expect(NOTE),
        None
    );
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(Some("acct"), Some("gateway"), None, None),
            &CloudflareGatewayOptions::default(),
        )
        .expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn prefers_environment_values_over_auth_and_config_options() {
    let options = CloudflareGatewayOptions {
        account_id: Some("auth-account".into()),
        gateway: Some("auth-gateway".into()),
        gateway_id: None,
        api_key: Some("auth-token".into()),
    };
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(
                Some("env-account"),
                Some("env-gateway"),
                Some("env-token"),
                None
            ),
            &options,
        )
        .expect(NOTE),
        Some(config("env-account", "env-gateway", "env-token"))
    );
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn accepts_a_gateway_id_option_as_the_gateway() {
    let options = CloudflareGatewayOptions {
        account_id: Some("auth-account".into()),
        gateway: None,
        gateway_id: Some("auth-gateway".into()),
        api_key: Some("auth-token".into()),
    };
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(&env(None, None, None, None), &options)
            .expect(NOTE),
        Some(config("auth-account", "auth-gateway", "auth-token"))
    );
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn falls_back_to_cf_aig_token_when_the_api_token_is_unset() {
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_config(
            &env(Some("acct"), Some("gateway"), None, Some("cf-aig-token")),
            &CloudflareGatewayOptions::default(),
        )
        .expect(NOTE),
        Some(config("acct", "gateway", "cf-aig-token"))
    );
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn prefers_the_metadata_option_over_the_legacy_header() {
    assert_eq!(
        CloudflareAiGatewayPlugin::resolve_metadata(
            Some(&json!({ "invoked_by": "test", "project": "opencode" })),
            Some(r#"{"invoked_by":"header"}"#),
        )
        .expect(NOTE),
        Some(json!({ "invoked_by": "test", "project": "opencode" }))
    );
}

#[test]
#[ignore = "porting: cloudflare-ai-gateway provider plugin not implemented"]
fn parses_the_legacy_cf_aig_metadata_header_when_metadata_is_absent() {
    let metadata = CloudflareAiGatewayPlugin::resolve_metadata(
        None,
        Some(r#"{"invoked_by":"header","project":"opencode"}"#),
    )
    .expect(NOTE);
    assert_eq!(
        metadata,
        Some(json!({ "invoked_by": "header", "project": "opencode" }))
    );
}
