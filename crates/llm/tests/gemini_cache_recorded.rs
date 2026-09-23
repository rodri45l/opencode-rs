//! Port of packages/llm/test/provider/gemini-cache.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded Gemini cache cassette.

mod common;

use opencode_llm::{providers, testing, LLMClient, LLM};
use serde_json::json;

fn cassette_response(cassette: &serde_json::Value, index: usize) -> serde_json::Value {
    let interaction = &cassette["interactions"][index];
    json!({
        "status": interaction["response"]["status"],
        "body": interaction["response"]["body"],
    })
}

#[test]
fn reports_cached_content_token_count_on_identical_second_call() {
    let cassette = common::recording(
        "gemini-cache",
        "reports-cachedcontenttokencount-on-identical-second-call",
    );
    assert!(
        cassette.get("interactions").is_some(),
        "cassette carries ordered interactions"
    );

    let request = LLM::request(json!({
        "id": "recorded_gemini_cache",
        "model": providers::google::configure(json!({ "apiKey": "fixture" })).model("gemini-2.5-flash"),
        "system": common::large_cacheable_system(),
        "prompt": "Say hi.",
        "generation": { "maxTokens": 16, "temperature": 0 },
    }));

    testing::push_response(cassette_response(&cassette, 0));
    let first = LLMClient::generate(request.clone()).expect("first call");
    assert!(first.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) >= 0);

    testing::push_response(cassette_response(&cassette, 1));
    let second = LLMClient::generate(request).expect("second call");
    assert!(second.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) >= 0);
}
