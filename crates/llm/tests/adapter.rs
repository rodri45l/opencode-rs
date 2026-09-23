//! Port of packages/llm/test/adapter.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the route pipeline (stream/generate/prepare).
//! The fake framing/protocol Effect plumbing is dropped; wire behaviour is kept.

use opencode_llm::{LLMClient, LLMResponse, LLM};
use serde_json::json;

fn request(route_id: &str) -> serde_json::Value {
    LLM::request(json!({
        "id": "req_1",
        "model": { "id": "fake-model", "provider": "fake-provider", "route": { "id": route_id }, "endpoint": { "baseURL": "https://fake.local" } },
        "prompt": "hello",
    }))
}

#[test]
#[ignore = "porting: route pipeline not implemented"]
fn stream_and_generate_use_the_route_pipeline() {
    let events = LLMClient::stream(request("fake")).expect("stream");
    let response = LLMClient::generate(request("fake")).expect("generate");
    let reduced = LLMResponse::from_events(events.clone())
        .expect("reduce")
        .expect("completed");

    assert_eq!(
        events.iter().map(|e| e["type"].clone()).collect::<Vec<_>>(),
        vec![json!("text-delta"), json!("finish")]
    );
    assert_eq!(response.events, events);
    assert_eq!(response.message, reduced["message"]);
    assert_eq!(response.usage, reduced["usage"]);
    assert_eq!(
        response.message["content"],
        json!([{ "type": "text", "text": "echo:{\"body\":\"hello\"}" }])
    );
}

#[test]
#[ignore = "porting: route pipeline not implemented"]
fn selects_routes_by_model_route_value() {
    let prepared = LLMClient::prepare(request("gemini-fake")).expect("prepare");

    assert_eq!(prepared.route, "gemini-fake");
}

#[test]
#[ignore = "porting: route pipeline not implemented"]
fn does_not_register_duplicate_route_ids_globally() {
    let prepared = LLMClient::prepare(request("fake")).expect("prepare");

    assert_eq!(prepared.body, json!({ "body": "late-default" }));
}
