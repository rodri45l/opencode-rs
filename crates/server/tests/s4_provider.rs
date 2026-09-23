//! Port of packages/opencode/test/provider/provider.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/provider/provider.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure synchronous unit tests — `parseModel`, `Provider.sort`, the
//! `reasoning_options` -> variants derivation (OpenAI / Google / Anthropic /
//! generic effort / toggle fallback / MERGE gateway), and `toPublicInfo` dropping
//! models with invalid (NaN) costs.
//! Dropped: the large `it.instance` surface that loads providers through config,
//! env, auth.json, models.dev, and the live SDK (defaultModel, closest,
//! getSmallModel, endpoint rewriting, plugin persistence, etc.).

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn parse_model(_input: &str) -> Result<Value, NotImplemented> {
    nope("provider")
}

fn provider_sort(_models: &[Value]) -> Result<Vec<Value>, NotImplemented> {
    nope("provider")
}

fn reasoning_variants(_reasoning_options: &Value, _npm: &str) -> Result<Value, NotImplemented> {
    nope("provider")
}

fn to_public_info(_models: &Value) -> Result<Value, NotImplemented> {
    nope("provider")
}

#[test]
#[ignore = "porting: provider not implemented"]
fn parse_model_correctly_parses_provider_model_string() {
    assert_eq!(
        parse_model("anthropic/claude-sonnet-4").unwrap(),
        json!({ "providerID": "anthropic", "modelID": "claude-sonnet-4" })
    );
}

#[test]
#[ignore = "porting: provider not implemented"]
fn parse_model_handles_model_ids_with_slashes() {
    assert_eq!(
        parse_model("openrouter/anthropic/claude-3-opus").unwrap(),
        json!({ "providerID": "openrouter", "modelID": "anthropic/claude-3-opus" })
    );
}

#[test]
#[ignore = "porting: provider not implemented"]
fn provider_sort_prioritizes_preferred_models() {
    let models = vec![
        json!({ "id": "random-model", "name": "Random" }),
        json!({ "id": "claude-sonnet-4-latest", "name": "Claude Sonnet 4" }),
        json!({ "id": "gpt-5-turbo", "name": "GPT-5 Turbo" }),
        json!({ "id": "other-model", "name": "Other" }),
    ];
    let sorted = provider_sort(&models).unwrap();
    let first = sorted[0]["id"].as_str().unwrap();
    let last = sorted[sorted.len() - 1]["id"].as_str().unwrap();
    assert!(first.contains("sonnet-4"));
    assert!(first.contains("latest"));
    assert!(!last.contains("gpt-5"));
    assert!(!last.contains("sonnet-4"));
}

#[test]
#[ignore = "porting: provider not implemented"]
fn models_dev_reasoning_options_replace_generated_variants() {
    let explicit = reasoning_variants(
        &json!([{ "type": "effort", "values": ["low"] }]),
        "@ai-sdk/openai",
    )
    .unwrap();
    assert_eq!(
        explicit,
        json!({
            "low": {
                "reasoningEffort": "low",
                "reasoningSummary": "auto",
                "include": ["reasoning.encrypted_content"]
            }
        })
    );

    let empty = reasoning_variants(&json!([]), "@ai-sdk/openai").unwrap();
    assert_eq!(empty, json!({}));
}

#[test]
#[ignore = "porting: provider not implemented"]
fn unsupported_reasoning_toggles_fall_back_to_generated_variants() {
    let fallback = reasoning_variants(&json!([{ "type": "toggle" }]), "@ai-sdk/openai").unwrap();
    let keys: Vec<_> = fallback.as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, vec!["none", "low", "medium", "high", "xhigh"]);
}

#[test]
#[ignore = "porting: provider not implemented"]
fn reasoning_options_are_npm_aware() {
    let google = reasoning_variants(
        &json!([{ "type": "effort", "values": ["high"] }]),
        "@ai-sdk/google",
    )
    .unwrap();
    assert_eq!(
        google,
        json!({ "high": { "thinkingConfig": { "includeThoughts": true, "thinkingLevel": "high" } } })
    );

    let anthropic = reasoning_variants(
        &json!([{ "type": "effort", "values": ["max"] }]),
        "@ai-sdk/anthropic",
    )
    .unwrap();
    assert_eq!(anthropic, json!({ "max": { "effort": "max" } }));
}

#[test]
#[ignore = "porting: provider not implemented"]
fn merge_gateway_exposes_declared_effort_variants() {
    let variants = reasoning_variants(
        &json!([{ "type": "effort", "values": ["none", "low", "medium", "high", "xhigh", "max"] }]),
        "merge-gateway-ai-sdk-provider",
    )
    .unwrap();
    assert_eq!(
        variants,
        json!({
            "none": { "reasoningEffort": "none" },
            "low": { "reasoningEffort": "low" },
            "medium": { "reasoningEffort": "medium" },
            "high": { "reasoningEffort": "high" },
            "xhigh": { "reasoningEffort": "xhigh" },
            "max": { "reasoningEffort": "max" }
        })
    );
}

#[test]
#[ignore = "porting: provider not implemented"]
fn public_provider_info_omits_invalid_models() {
    let models = json!({
        "valid": { "id": "valid", "name": "Valid", "cost": { "input": 1, "output": 1 } },
        "invalid": { "id": "invalid", "name": "Invalid", "cost": { "input": null, "output": 1 } }
    });
    let result = to_public_info(&models).unwrap();
    assert!(result.get("valid").is_some());
    assert!(result.get("invalid").is_none());
}
