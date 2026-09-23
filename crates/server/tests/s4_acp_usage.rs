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

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn build_usage(_tokens: &Value) -> Result<Value, NotImplemented> {
    nope("acp usage")
}

fn latest_assistant_message(_messages: &Value) -> Result<Option<Value>, NotImplemented> {
    nope("acp usage")
}

fn total_session_cost(_messages: &Value) -> Result<f64, NotImplemented> {
    nope("acp usage")
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
#[ignore = "porting: acp usage not implemented"]
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
#[ignore = "porting: acp usage not implemented"]
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
#[ignore = "porting: acp usage not implemented"]
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
#[ignore = "porting: acp usage not implemented"]
fn calculates_total_session_cost_from_assistant_messages() {
    let messages = json!([assistant(1.25), { "info": { "role": "user" } }, assistant(2.5)]);
    assert_eq!(total_session_cost(&messages).unwrap(), 3.75);
}
