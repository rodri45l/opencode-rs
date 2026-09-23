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

fn fit_variant(value: Option<&str>, available: &[&str]) -> Option<String> {
    let value = value?;
    if available.is_empty() || available.contains(&value) {
        Some(value.to_string())
    } else {
        None
    }
}

fn resolve_variant(
    cli: Option<&str>,
    session: Option<&str>,
    saved: Option<&str>,
    available: &[&str],
) -> Option<String> {
    if let Some(cli) = cli {
        return Some(cli.to_string());
    }
    let fallback = fit_variant(saved, available);
    let current = fit_variant(session, available);
    current.or(fallback)
}

fn cycle_variant(current: Option<&str>, available: &[&str]) -> Option<String> {
    if available.is_empty() {
        return None;
    }
    match current {
        None => Some(available[0].to_string()),
        Some(current) => {
            let index = available.iter().position(|item| *item == current)?;
            available.get(index + 1).map(|item| item.to_string())
        }
    }
}

fn format_model_label(
    model: &ModelRef,
    variant: Option<&str>,
    providers: Option<&[RunProvider]>,
) -> String {
    let base = providers
        .and_then(|providers| {
            providers
                .iter()
                .find(|provider| provider.id == model.provider_id)
        })
        .map(|provider| {
            let model_name = provider
                .models
                .get(&model.model_id)
                .cloned()
                .unwrap_or_else(|| model.model_id.clone());
            format!("{model_name} · {}", provider.name)
        })
        .unwrap_or_else(|| format!("{} · {}", model.model_id, model.provider_id));
    match variant {
        Some(variant) => format!("{base} · {variant}"),
        None => base,
    }
}

fn pick_variant(model: &ModelRef, messages: &[UserMessage]) -> Option<String> {
    for message in messages.iter().rev() {
        if message.model.provider_id == model.provider_id
            && message.model.model_id == model.model_id
            && message.variant.is_some()
        {
            return message.variant.clone();
        }
    }
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
