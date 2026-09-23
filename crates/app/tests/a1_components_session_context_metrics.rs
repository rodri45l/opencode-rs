//! Port of packages/app/src/components/session/session-context-metrics.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    role: String,
    provider_id: String,
    model_id: String,
    cost: f64,
    tokens: Option<Tokens>,
}

#[derive(Clone, Debug, PartialEq)]
struct Tokens {
    input: i64,
    output: i64,
    reasoning: i64,
    cache_read: i64,
    cache_write: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Provider {
    id: String,
    name: Option<String>,
    models: BTreeMap<String, ModelInfo>,
}

#[derive(Clone, Debug, PartialEq)]
struct ModelInfo {
    name: String,
    context_limit: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Context {
    message: Message,
    total: i64,
    input: i64,
    usage: Option<f64>,
    provider_label: String,
    model_label: String,
    limit: Option<i64>,
}

// Local stub (fast wave): real module lands later.
fn get_session_context(_messages: &[Message], _providers: &[Provider]) -> Option<Context> {
    None
}

#[allow(clippy::too_many_arguments)]
fn assistant(
    id: &str,
    input: i64,
    output: i64,
    reasoning: i64,
    read: i64,
    write: i64,
    cost: f64,
    provider_id: &str,
    model_id: &str,
) -> Message {
    Message {
        id: id.into(),
        role: "assistant".into(),
        provider_id: provider_id.into(),
        model_id: model_id.into(),
        cost,
        tokens: Some(Tokens {
            input,
            output,
            reasoning,
            cache_read: read,
            cache_write: write,
        }),
    }
}

fn user(id: &str) -> Message {
    Message {
        id: id.into(),
        role: "user".into(),
        provider_id: String::new(),
        model_id: String::new(),
        cost: 0.0,
        tokens: None,
    }
}

fn openai_provider() -> Provider {
    let mut models = BTreeMap::new();
    models.insert(
        "gpt-4.1".to_string(),
        ModelInfo {
            name: "GPT-4.1".into(),
            context_limit: 1000,
        },
    );
    Provider {
        id: "openai".into(),
        name: Some("OpenAI".into()),
        models,
    }
}

#[test]
#[ignore = "porting: components/session-context-metrics not implemented"]
fn computes_token_totals_and_usage_from_latest_assistant_with_tokens() {
    let messages = vec![
        user("u1"),
        assistant("a1", 600, 200, 100, 50, 50, 0.5, "openai", "gpt-4.1"),
        assistant("a2", 300, 100, 50, 25, 25, 1.25, "openai", "gpt-4.1"),
    ];
    let ctx = get_session_context(&messages, &[openai_provider()]);
    assert_eq!(
        ctx.as_ref().map(|c| c.message.id.clone()),
        Some("a2".to_string())
    );
    assert_eq!(ctx.as_ref().map(|c| c.total), Some(500));
    assert_eq!(ctx.as_ref().map(|c| c.input), Some(300));
    assert_eq!(ctx.as_ref().map(|c| c.usage), Some(Some(50.0)));
    assert_eq!(
        ctx.as_ref().map(|c| c.provider_label.clone()),
        Some("OpenAI".to_string())
    );
    assert_eq!(
        ctx.as_ref().map(|c| c.model_label.clone()),
        Some("GPT-4.1".to_string())
    );
}

#[test]
#[ignore = "porting: components/session-context-metrics not implemented"]
fn preserves_fallback_labels_and_null_usage_when_model_metadata_is_missing() {
    let messages = vec![assistant("a1", 40, 10, 0, 0, 0, 0.1, "p-1", "m-1")];
    let providers = vec![Provider {
        id: "p-1".into(),
        name: None,
        models: BTreeMap::new(),
    }];
    let ctx = get_session_context(&messages, &providers);
    assert_eq!(
        ctx.as_ref().map(|c| c.provider_label.clone()),
        Some("p-1".to_string())
    );
    assert_eq!(
        ctx.as_ref().map(|c| c.model_label.clone()),
        Some("m-1".to_string())
    );
    assert_eq!(ctx.as_ref().and_then(|c| c.limit), None);
    assert_eq!(ctx.as_ref().and_then(|c| c.usage), None);
}

#[test]
#[ignore = "porting: components/session-context-metrics not implemented"]
fn recomputes_when_message_array_is_mutated_in_place() {
    let mut messages = vec![assistant(
        "a1", 10, 10, 10, 10, 10, 0.25, "openai", "gpt-4.1",
    )];
    let providers = vec![Provider {
        id: "openai".into(),
        name: None,
        models: BTreeMap::new(),
    }];
    let one = get_session_context(&messages, &providers);
    messages.push(assistant("a2", 100, 20, 0, 0, 0, 0.75, "openai", "gpt-4.1"));
    let two = get_session_context(&messages, &providers);
    assert_eq!(
        one.as_ref().map(|c| c.message.id.clone()),
        Some("a1".to_string())
    );
    assert_eq!(
        two.as_ref().map(|c| c.message.id.clone()),
        Some("a2".to_string())
    );
}

#[test]
#[ignore = "porting: components/session-context-metrics not implemented"]
fn returns_undefined_when_inputs_are_undefined() {
    assert_eq!(get_session_context(&[], &[]), None);
}
