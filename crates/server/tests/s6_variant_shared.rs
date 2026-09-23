//! Port of packages/opencode/test/cli/run/variant.shared.test.ts (upstream 18ef3cc).
//!
//! Only the pure variant-selection helpers are in scope; the `it.live` runtime
//! cases (saved-variant read/write through the app fs layer) are dropped as
//! live-runtime. RED-first: `cli/cmd/run/variant.shared` is not implemented in
//! this crate, so behaviour is pinned against local typed stubs.

#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelRef {
    provider_id: String,
    model_id: String,
}

fn model() -> ModelRef {
    ModelRef {
        provider_id: "openai".to_string(),
        model_id: "gpt-5".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunProvider {
    id: String,
    name: String,
    models: BTreeMap<String, String>,
}

fn providers() -> Vec<RunProvider> {
    let mut models = BTreeMap::new();
    models.insert("gpt-5".to_string(), "GPT-5".to_string());
    vec![RunProvider {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models,
    }]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UserMessage {
    model: ModelRef,
    variant: Option<String>,
}

fn resolve_variant(
    _cli: Option<&str>,
    _session: Option<&str>,
    _saved: Option<&str>,
    _available: &[&str],
) -> Option<String> {
    None
}

fn cycle_variant(_current: Option<&str>, _available: &[&str]) -> Option<String> {
    None
}

fn format_model_label(
    _model: &ModelRef,
    _variant: Option<&str>,
    _providers: Option<&[RunProvider]>,
) -> String {
    String::new()
}

fn pick_variant(_model: &ModelRef, _messages: &[UserMessage]) -> Option<String> {
    None
}

fn msg(provider: &str, model_id: &str, variant: &str) -> UserMessage {
    UserMessage {
        model: ModelRef {
            provider_id: provider.to_string(),
            model_id: model_id.to_string(),
        },
        variant: Some(variant.to_string()),
    }
}

#[test]
#[ignore = "porting: cli run variant selection not implemented"]
fn prefers_cli_then_session_then_saved_variants() {
    assert_eq!(
        resolve_variant(Some("max"), Some("high"), Some("low"), &["low", "high"]),
        Some("max".to_string())
    );
    assert_eq!(
        resolve_variant(None, Some("high"), Some("low"), &["low", "high"]),
        Some("high".to_string())
    );
    assert_eq!(
        resolve_variant(None, Some("missing"), Some("low"), &["low", "high"]),
        Some("low".to_string())
    );
}

#[test]
#[ignore = "porting: cli run variant selection not implemented"]
fn cycles_through_variants_and_back_to_default() {
    assert_eq!(
        cycle_variant(None, &["low", "high"]),
        Some("low".to_string())
    );
    assert_eq!(
        cycle_variant(Some("low"), &["low", "high"]),
        Some("high".to_string())
    );
    assert_eq!(cycle_variant(Some("high"), &["low", "high"]), None);
    assert_eq!(cycle_variant(None, &[]), None);
}

#[test]
#[ignore = "porting: cli run variant selection not implemented"]
fn formats_model_labels() {
    assert_eq!(format_model_label(&model(), None, None), "gpt-5 · openai");
    assert_eq!(
        format_model_label(&model(), Some("high"), None),
        "gpt-5 · openai · high"
    );
    assert_eq!(
        format_model_label(&model(), None, Some(&providers())),
        "GPT-5 · OpenAI"
    );
    assert_eq!(
        format_model_label(&model(), Some("high"), Some(&providers())),
        "GPT-5 · OpenAI · high"
    );
}

#[test]
#[ignore = "porting: cli run variant selection not implemented"]
fn picks_the_latest_matching_variant_from_raw_session_messages() {
    let messages = vec![
        msg("openai", "gpt-5", "high"),
        msg("anthropic", "sonnet", "max"),
        msg("openai", "gpt-5", "minimal"),
    ];
    assert_eq!(
        pick_variant(&model(), &messages),
        Some("minimal".to_string())
    );
}
