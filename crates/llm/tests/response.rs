//! Port of packages/llm/test/response.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `LLMResponse` reducer.

use opencode_llm::{LLMEvent, LLMResponse};
use serde_json::{json, Value};

fn reduce(events: &[Value]) -> Value {
    events.iter().fold(LLMResponse::empty(), |state, event| {
        LLMResponse::reduce(state, event.clone())
    })
}

#[test]
#[ignore = "porting: llm response reducer not implemented"]
fn assembles_interleaved_reasoning_and_text_with_end_metadata() {
    let events = vec![
        json!({ "type": "reasoning-start", "id": "r1" }),
        json!({ "type": "reasoning-delta", "id": "r1", "text": "I should " }),
        json!({ "type": "text-start", "id": "t1" }),
        json!({ "type": "reasoning-delta", "id": "r1", "text": "compare..." }),
        json!({ "type": "reasoning-end", "id": "r1", "providerMetadata": { "anthropic": { "signature": "sig" } } }),
        json!({ "type": "text-delta", "id": "t1", "text": "Answer" }),
        json!({ "type": "text-end", "id": "t1" }),
        json!({ "type": "finish", "reason": "stop", "usage": { "outputTokens": 5 } }),
    ];

    let response = LLMResponse::from_events(events.clone())
        .expect("reduce")
        .expect("completed response");

    assert_eq!(response["finishReason"], "stop");
    assert_eq!(response["usage"]["outputTokens"], 5);
    assert_eq!(response["events"], json!(events));
    assert_eq!(
        response["message"]["content"],
        json!([
            { "type": "reasoning", "text": "I should compare...", "providerMetadata": { "anthropic": { "signature": "sig" } } },
            { "type": "text", "text": "Answer" }
        ])
    );
    let finish_count = response["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter(|event| LLMEvent::is_finish(event))
        .count();
    assert_eq!(finish_count, 1);
}

#[test]
#[ignore = "porting: llm response reducer not implemented"]
fn preserves_partial_content_without_completing_a_failed_stream() {
    let state = reduce(&[
        json!({ "type": "text-start", "id": "t1" }),
        json!({ "type": "text-delta", "id": "t1", "text": "partial" }),
    ]);

    assert!(LLMResponse::complete(&state).is_none());
    assert_eq!(
        state["message"]["content"],
        json!([{ "type": "text", "text": "partial" }])
    );
}

#[test]
#[ignore = "porting: llm response reducer not implemented"]
fn does_not_complete_ended_content_without_a_terminal_finish() {
    let state = reduce(&[
        json!({ "type": "text-start", "id": "t1" }),
        json!({ "type": "text-delta", "id": "t1", "text": "partial" }),
        json!({ "type": "text-end", "id": "t1" }),
    ]);

    assert!(LLMResponse::complete(&state).is_none());
    assert_eq!(
        state["message"]["content"],
        json!([{ "type": "text", "text": "partial" }])
    );
}

#[test]
#[ignore = "porting: llm response reducer not implemented"]
fn uses_terminal_usage_and_keeps_prior_usage_when_finish_omits_it() {
    let with_finish_usage = LLMResponse::from_events(vec![
        LLMEvent::step_finish(0, "stop", Some(json!({ "inputTokens": 3 }))),
        LLMEvent::finish("stop", Some(json!({ "outputTokens": 2 }))),
    ])
    .expect("reduce")
    .expect("completed");

    let without_finish_usage = LLMResponse::from_events(vec![
        LLMEvent::step_finish(0, "stop", Some(json!({ "inputTokens": 3 }))),
        LLMEvent::finish("stop", None),
    ])
    .expect("reduce")
    .expect("completed");

    assert_eq!(with_finish_usage["usage"]["outputTokens"], 2);
    assert_eq!(without_finish_usage["usage"]["inputTokens"], 3);
}

#[test]
#[ignore = "porting: llm response reducer not implemented"]
fn assembles_tool_call_content_only_after_the_completed_tool_call_event() {
    let pending = reduce(&[
        json!({ "type": "tool-input-start", "id": "call_1", "name": "lookup" }),
        json!({ "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": "{\"query\"" }),
    ]);

    assert_eq!(pending["message"]["content"], json!([]));
    assert_eq!(pending["toolInputs"]["call_1"]["text"], "{\"query\"");

    let events = vec![
        json!({ "type": "tool-input-start", "id": "call_1", "name": "lookup" }),
        json!({ "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": "{\"query\"" }),
        json!({ "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": ":\"weather\"}" }),
        json!({ "type": "tool-input-end", "id": "call_1", "name": "lookup" }),
        json!({ "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "weather" } }),
        json!({ "type": "finish", "reason": "tool-calls" }),
    ];
    let response = LLMResponse::from_events(events)
        .expect("reduce")
        .expect("completed");

    assert_eq!(
        response["message"]["content"],
        json!([{ "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "weather" } }])
    );
}
