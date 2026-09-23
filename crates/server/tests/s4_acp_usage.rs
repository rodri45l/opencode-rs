//! Port of packages/opencode/test/acp/usage.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/usage.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure helpers `buildUsage`, `latestAssistantMessage`, and
//! `totalSessionCost`.
//! Dropped: the Effect-layer `contextLimit` / `sendUpdate` cases (need the
//! Provider runtime, message loader, and live ACP connection).

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn build_usage(tokens: &Value) -> Result<Value, NotImplemented> {
    let number = |key: &str| tokens.get(key).and_then(Value::as_i64).unwrap_or(0);
    let input = number("input");
    let output = number("output");
    let reasoning = number("reasoning");
    let read = tokens
        .get("cache")
        .and_then(|cache| cache.get("read"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let write = tokens
        .get("cache")
        .and_then(|cache| cache.get("write"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let mut usage = serde_json::Map::new();
    usage.insert("inputTokens".to_string(), json!(input));
    usage.insert("outputTokens".to_string(), json!(output));
    if reasoning != 0 {
        usage.insert("thoughtTokens".to_string(), json!(reasoning));
    }
    if read != 0 {
        usage.insert("cachedReadTokens".to_string(), json!(read));
    }
    if write != 0 {
        usage.insert("cachedWriteTokens".to_string(), json!(write));
    }
    usage.insert(
        "totalTokens".to_string(),
        json!(input + output + reasoning + read + write),
    );
    Ok(Value::Object(usage))
}

fn latest_assistant_message(messages: &Value) -> Result<Option<Value>, NotImplemented> {
    let latest = messages
        .as_array()
        .map(|items| {
            items
                .iter()
                .rfind(|message| message["info"]["role"] == json!("assistant"))
                .cloned()
        })
        .unwrap_or(None);
    Ok(latest)
}

fn total_session_cost(messages: &Value) -> Result<f64, NotImplemented> {
    let total = messages
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|message| message["info"]["role"] == json!("assistant"))
                .filter_map(|message| message["info"]["cost"].as_f64())
                .sum()
        })
        .unwrap_or(0.0);
    Ok(total)
}

fn assistant(cost: f64) -> Value {
    json!({
        "info": {
            "role": "assistant",
            "providerID": "anthropic",
            "modelID": "claude-sonnet",
            "cost": cost,
            "tokens": { "input": 10, "output": 20, "reasoning": 0, "cache": { "read": 0, "write": 0 } }
        }
    })
}

#[test]
fn builds_acp_usage_from_assistant_token_shape() {
    assert_eq!(
        build_usage(&json!({
            "input": 100,
            "output": 40,
            "reasoning": 7,
            "cache": { "read": 11, "write": 13 }
        }))
        .unwrap(),
        json!({
            "inputTokens": 100,
            "outputTokens": 40,
            "thoughtTokens": 7,
            "cachedReadTokens": 11,
            "cachedWriteTokens": 13,
            "totalTokens": 171
        })
    );
}

#[test]
fn omits_optional_token_fields_when_they_are_zero() {
    assert_eq!(
        build_usage(&json!({
            "input": 3,
            "output": 4,
            "reasoning": 0,
            "cache": { "read": 0, "write": 0 }
        }))
        .unwrap(),
        json!({ "inputTokens": 3, "outputTokens": 4, "totalTokens": 7 })
    );
}

#[test]
fn finds_the_latest_assistant_message() {
    let messages = json!([
        assistant(1.0),
        { "info": { "role": "user" } },
        assistant(2.0)
    ]);
    let latest = latest_assistant_message(&messages).unwrap().unwrap();
    assert_eq!(latest["info"]["cost"], json!(2.0));
}

#[test]
fn calculates_total_session_cost_from_assistant_messages() {
    let messages = json!([assistant(1.25), { "info": { "role": "user" } }, assistant(2.5)]);
    assert_eq!(total_session_cost(&messages).unwrap(), 3.75);
}
