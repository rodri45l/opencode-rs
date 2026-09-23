//! Port of packages/core/test/plugin/provider-amazon-bedrock.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Bedrock plugin binds only to `@ai-sdk/amazon-bedrock`
//! and its `mantle` subpath, applies the legacy cross-region inference prefix
//! matrix, resolves region as config over `AWS_REGION` over `us-east-1`,
//! resolves base URL as `endpoint` over `baseURL` over the region default, and
//! selects the Mantle `responses`/`chat` accessor by model id. Re-derived: the
//! `AISDK`/`Catalog`/`PluginHost` service wiring, SigV4 request signing, and
//! environment credential loading are dropped; the pure decisions remain.

use opencode_core::provider_amazon_bedrock::{AmazonBedrockPlugin, DEFAULT_REGION};
use opencode_core::provider_sdk_plugins::LanguageSelector;

const NOTE: &str = "porting: amazon-bedrock provider plugin not implemented";

#[test]
#[ignore = "porting: amazon-bedrock provider plugin not implemented"]
fn binds_only_to_the_bedrock_package_and_its_mantle_subpath() {
    assert!(AmazonBedrockPlugin::matches_package("@ai-sdk/amazon-bedrock").expect(NOTE));
    assert!(AmazonBedrockPlugin::matches_package("@ai-sdk/amazon-bedrock/mantle").expect(NOTE));
    assert!(!AmazonBedrockPlugin::matches_package("@ai-sdk/amazon-bedrock/anthropic").expect(NOTE));
    assert!(!AmazonBedrockPlugin::matches_package("@ai-sdk/openai-compatible").expect(NOTE));
}

#[test]
#[ignore = "porting: amazon-bedrock provider plugin not implemented"]
fn resolves_region_as_config_over_environment_over_default() {
    assert_eq!(
        AmazonBedrockPlugin::resolve_region(Some("eu-west-1"), Some("us-east-1")).expect(NOTE),
        "eu-west-1"
    );
    assert_eq!(
        AmazonBedrockPlugin::resolve_region(None, Some("eu-west-1")).expect(NOTE),
        "eu-west-1"
    );
    assert_eq!(
        AmazonBedrockPlugin::resolve_region(None, None).expect(NOTE),
        DEFAULT_REGION
    );
}

#[test]
#[ignore = "porting: amazon-bedrock provider plugin not implemented"]
fn resolves_base_url_as_endpoint_over_base_url_over_region_default() {
    assert_eq!(
        AmazonBedrockPlugin::resolve_base_url(
            Some("https://endpoint.example"),
            Some("https://base.example"),
            "us-east-1"
        )
        .expect(NOTE),
        "https://endpoint.example"
    );
    assert_eq!(
        AmazonBedrockPlugin::resolve_base_url(None, Some("https://base.example"), "us-east-1")
            .expect(NOTE),
        "https://base.example"
    );
    assert_eq!(
        AmazonBedrockPlugin::resolve_base_url(None, None, "eu-west-1").expect(NOTE),
        "https://bedrock-runtime.eu-west-1.amazonaws.com"
    );
    assert_eq!(
        AmazonBedrockPlugin::resolve_base_url(None, None, DEFAULT_REGION).expect(NOTE),
        "https://bedrock-runtime.us-east-1.amazonaws.com"
    );
}

#[test]
#[ignore = "porting: amazon-bedrock provider plugin not implemented"]
fn applies_the_full_legacy_cross_region_prefix_matrix() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "us-east-1",
            "amazon.nova-micro-v1:0",
            "us.amazon.nova-micro-v1:0",
        ),
        (
            "us-east-1",
            "amazon.nova-lite-v1:0",
            "us.amazon.nova-lite-v1:0",
        ),
        (
            "us-east-1",
            "amazon.nova-pro-v1:0",
            "us.amazon.nova-pro-v1:0",
        ),
        (
            "us-east-1",
            "amazon.nova-premier-v1:0",
            "us.amazon.nova-premier-v1:0",
        ),
        (
            "us-east-1",
            "amazon.nova-2-lite-v1:0",
            "us.amazon.nova-2-lite-v1:0",
        ),
        (
            "us-east-1",
            "anthropic.claude-sonnet-4-5",
            "us.anthropic.claude-sonnet-4-5",
        ),
        ("us-east-1", "deepseek.r1-v1:0", "us.deepseek.r1-v1:0"),
        ("us-east-1", "us.deepseek.r1-v1:0", "us.deepseek.r1-v1:0"),
        ("us-east-1", "deepseek.v3.2", "deepseek.v3.2"),
        (
            "us-east-1",
            "arn:aws:bedrock:us-east-1::foundation-model/deepseek.v3.2",
            "arn:aws:bedrock:us-east-1::foundation-model/deepseek.v3.2",
        ),
        (
            "us-gov-west-1",
            "anthropic.claude-sonnet-4-5",
            "anthropic.claude-sonnet-4-5",
        ),
        (
            "us-east-1",
            "cohere.command-r-plus-v1:0",
            "cohere.command-r-plus-v1:0",
        ),
        (
            "eu-west-1",
            "anthropic.claude-sonnet-4-5",
            "eu.anthropic.claude-sonnet-4-5",
        ),
        (
            "eu-west-2",
            "amazon.nova-lite-v1:0",
            "eu.amazon.nova-lite-v1:0",
        ),
        (
            "eu-west-3",
            "amazon.nova-micro-v1:0",
            "eu.amazon.nova-micro-v1:0",
        ),
        (
            "eu-north-1",
            "meta.llama3-70b-instruct-v1:0",
            "eu.meta.llama3-70b-instruct-v1:0",
        ),
        (
            "eu-central-1",
            "mistral.pixtral-large-v1:0",
            "eu.mistral.pixtral-large-v1:0",
        ),
        (
            "eu-south-1",
            "anthropic.claude-sonnet-4-5",
            "eu.anthropic.claude-sonnet-4-5",
        ),
        (
            "eu-south-2",
            "anthropic.claude-sonnet-4-5",
            "eu.anthropic.claude-sonnet-4-5",
        ),
        (
            "eu-central-2",
            "anthropic.claude-sonnet-4-5",
            "anthropic.claude-sonnet-4-5",
        ),
        (
            "eu-west-1",
            "cohere.command-r-plus-v1:0",
            "cohere.command-r-plus-v1:0",
        ),
        (
            "ap-southeast-2",
            "anthropic.claude-sonnet-4-5",
            "au.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-southeast-4",
            "anthropic.claude-haiku-v1:0",
            "au.anthropic.claude-haiku-v1:0",
        ),
        (
            "ap-southeast-2",
            "anthropic.claude-opus-4",
            "apac.anthropic.claude-opus-4",
        ),
        (
            "ap-northeast-1",
            "anthropic.claude-sonnet-4-5",
            "jp.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-northeast-1",
            "amazon.nova-pro-v1:0",
            "jp.amazon.nova-pro-v1:0",
        ),
        (
            "ap-south-1",
            "anthropic.claude-sonnet-4-5",
            "apac.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-south-1",
            "amazon.nova-lite-v1:0",
            "apac.amazon.nova-lite-v1:0",
        ),
        (
            "ca-central-1",
            "anthropic.claude-sonnet-4-5",
            "anthropic.claude-sonnet-4-5",
        ),
        (
            "us-east-1",
            "global.anthropic.claude-sonnet-4-5",
            "global.anthropic.claude-sonnet-4-5",
        ),
        (
            "us-east-1",
            "us.anthropic.claude-sonnet-4-5",
            "us.anthropic.claude-sonnet-4-5",
        ),
        (
            "eu-west-1",
            "eu.anthropic.claude-sonnet-4-5",
            "eu.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-northeast-1",
            "jp.anthropic.claude-sonnet-4-5",
            "jp.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-south-1",
            "apac.anthropic.claude-sonnet-4-5",
            "apac.anthropic.claude-sonnet-4-5",
        ),
        (
            "ap-southeast-2",
            "au.anthropic.claude-sonnet-4-5",
            "au.anthropic.claude-sonnet-4-5",
        ),
    ];

    for (region, model_id, expected) in cases {
        assert_eq!(
            AmazonBedrockPlugin::apply_language_prefix(region, model_id).expect(NOTE),
            *expected,
            "region {region} model {model_id}"
        );
    }
}

#[test]
#[ignore = "porting: amazon-bedrock provider plugin not implemented"]
fn selects_mantle_accessors_without_cross_region_prefixes() {
    assert_eq!(
        AmazonBedrockPlugin::select_mantle_accessor("openai.gpt-5.5").expect(NOTE),
        LanguageSelector::Responses
    );
    assert_eq!(
        AmazonBedrockPlugin::select_mantle_accessor("openai.gpt-oss-safeguard-120b").expect(NOTE),
        LanguageSelector::Chat
    );
}
