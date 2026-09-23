//! Port of packages/llm/test/schema.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference `llm schema` and `LLM.Usage` suites.

use opencode_llm::{ContentPart, LLMEvent, LLMRequest, ProviderShared, Usage};
use serde_json::json;

#[test]
fn decodes_a_minimal_request() {
    let input = json!({
        "id": "req_1",
        "model": { "id": "fake-model", "provider": "fake-provider", "route": { "id": "openai-chat" } },
        "system": [{ "type": "text", "text": "You are terse." }],
        "messages": [{ "role": "user", "content": [{ "type": "text", "text": "hi" }] }],
        "tools": [],
        "generation": {},
    });

    let decoded = LLMRequest::input(input);

    assert_eq!(decoded["id"], "req_1");
    assert_eq!(decoded["messages"][0]["content"][0]["type"], "text");
}

#[test]
fn accepts_custom_route_ids() {
    let decoded = LLMRequest::input(json!({
        "model": { "id": "fake-model", "provider": "fake-provider", "route": { "id": "openai-responses" } },
        "system": [],
        "messages": [],
        "tools": [],
        "generation": {},
    }));

    assert_eq!(decoded["model"]["route"]["id"], "openai-responses");
}

#[test]
fn finish_constructors_accept_usage_input() {
    let step = LLMEvent::step_finish(0, "stop", Some(json!({ "inputTokens": 1 })));
    assert_eq!(step["usage"]["inputTokens"], 1);

    let finish = LLMEvent::finish("stop", Some(json!({ "outputTokens": 2 })));
    assert_eq!(finish["usage"]["outputTokens"], 2);
}

#[test]
fn content_part_tagged_union_exposes_guards() {
    assert!(ContentPart::guards_text(
        &json!({ "type": "text", "text": "hi" })
    ));
    assert!(!ContentPart::guards_media(
        &json!({ "type": "text", "text": "hi" })
    ));
}

#[test]
fn subtract_tokens_clamps_non_sensical_breakdowns_to_zero() {
    assert_eq!(ProviderShared::subtract_tokens(Some(5), Some(3)), Some(2));
    assert_eq!(ProviderShared::subtract_tokens(Some(5), Some(10)), Some(0));
    assert_eq!(ProviderShared::subtract_tokens(Some(5), None), Some(5));
    assert_eq!(ProviderShared::subtract_tokens(None, Some(3)), None);
    assert_eq!(ProviderShared::subtract_tokens(None, None), None);
}

#[test]
fn sum_tokens_returns_none_only_when_every_input_is_absent() {
    assert_eq!(
        ProviderShared::sum_tokens(&[Some(1), Some(2), Some(3)]),
        Some(6)
    );
    assert_eq!(
        ProviderShared::sum_tokens(&[Some(1), None, Some(3)]),
        Some(4)
    );
    assert_eq!(ProviderShared::sum_tokens(&[None, None, None]), None);
    assert_eq!(ProviderShared::sum_tokens(&[]), None);
}

#[test]
fn visible_output_tokens_clamps_reasoning_over_output_to_zero() {
    assert_eq!(
        Usage::new(json!({ "outputTokens": 10, "reasoningTokens": 4 })).visible_output_tokens(),
        6
    );
    assert_eq!(
        Usage::new(json!({ "outputTokens": 10 })).visible_output_tokens(),
        10
    );
    assert_eq!(
        Usage::new(json!({ "outputTokens": 4, "reasoningTokens": 10 })).visible_output_tokens(),
        0
    );
    assert_eq!(Usage::new(json!({})).visible_output_tokens(), 0);
}
