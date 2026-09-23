//! Port of packages/opencode/test/provider/amazon-bedrock.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/provider/provider.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure cross-region inference-profile prefix detection
//! (`global.`/`us.`/`eu.`/`jp.`/`apac.`/`au.`), including that non-prefixed and
//! amazon/cohere model ids are not treated as prefixed.
//! Dropped: the `it.instance` provider-loading surface (region/profile/endpoint
//! precedence, auth.json bearer tokens, AWS web-identity autoload, Mantle routing,
//! DeepSeek identifier preservation). Those need config/env/auth and the live SDK.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn has_cross_region_prefix(model_id: &str) -> Result<bool, NotImplemented> {
    Ok(["global.", "us.", "eu.", "jp.", "apac.", "au."]
        .iter()
        .any(|prefix| model_id.starts_with(prefix)))
}

#[test]
fn should_detect_global_prefix() {
    assert!(has_cross_region_prefix("global.anthropic.claude-opus-4-5-20251101-v1:0").unwrap());
}

#[test]
fn should_detect_us_prefix() {
    assert!(has_cross_region_prefix("us.anthropic.claude-opus-4-5-20251101-v1:0").unwrap());
}

#[test]
fn should_detect_eu_prefix() {
    assert!(has_cross_region_prefix("eu.anthropic.claude-opus-4-5-20251101-v1:0").unwrap());
}

#[test]
fn should_detect_jp_prefix() {
    assert!(has_cross_region_prefix("jp.anthropic.claude-sonnet-4-20250514-v1:0").unwrap());
}

#[test]
fn should_detect_apac_prefix() {
    assert!(has_cross_region_prefix("apac.anthropic.claude-sonnet-4-20250514-v1:0").unwrap());
}

#[test]
fn should_detect_au_prefix() {
    assert!(has_cross_region_prefix("au.anthropic.claude-sonnet-4-5-20250929-v1:0").unwrap());
}

#[test]
fn should_not_detect_prefix_for_non_prefixed_model() {
    assert!(!has_cross_region_prefix("anthropic.claude-opus-4-5-20251101-v1:0").unwrap());
}

#[test]
fn should_not_detect_prefix_for_amazon_nova_models() {
    assert!(!has_cross_region_prefix("amazon.nova-pro-v1:0").unwrap());
}

#[test]
fn should_not_detect_prefix_for_cohere_models() {
    assert!(!has_cross_region_prefix("cohere.command-r-plus-v1:0").unwrap());
}
