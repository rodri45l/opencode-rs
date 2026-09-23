//! Port of packages/llm/test/provider/openai-responses-cache.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded OpenAI Responses cache cassette.

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
fn reports_cached_tokens_on_identical_second_call() {
    let cassette = common::recording(
        "openai-responses-cache",
        "reports-cached-tokens-on-identical-second-call",
    );
    assert!(
        cassette.get("interactions").is_some(),
        "cassette carries ordered interactions"
    );

    let request = LLM::request(json!({
        "id": "recorded_openai_responses_cache",
        "model": providers::openai::configure(json!({ "apiKey": "fixture" })).responses("gpt-4.1-mini"),
        "system": common::large_cacheable_system(),
        "prompt": "Say hi.",
        "generation": { "maxTokens": 16, "temperature": 0 },
        "providerOptions": { "openai": { "promptCacheKey": "recorded-cache-test" } },
    }));

    testing::push_response(cassette_response(&cassette, 0));
    let first = LLMClient::generate(request.clone()).expect("first call");
    assert!(first.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) >= 0);

    testing::push_response(cassette_response(&cassette, 1));
    let second = LLMClient::generate(request).expect("second call");
    assert!(second.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) > 0);
}
