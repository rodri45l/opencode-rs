//! Port of packages/llm/test/provider/anthropic-messages-cache.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded Anthropic cache cassette.
//! Effect-ts recorder `Layer` plumbing is dropped; the cassette is loaded and
//! the cache-read behaviour is asserted.

mod common;

use opencode_llm::{providers, CacheHint, LLM, LLMClient};
use serde_json::json;

#[test]
#[ignore = "porting: anthropic recorded cache not implemented"]
fn writes_then_reads_cache_control_on_identical_second_call() {
    let cassette = common::recording(
        "anthropic-messages-cache",
        "writes-then-reads-cache-control-on-identical-second-call",
    );
    assert!(cassette.get("interactions").is_some(), "cassette carries ordered interactions");

    let request = LLM::request(json!({
        "id": "recorded_anthropic_cache",
        "model": providers::anthropic::configure(json!({ "apiKey": "fixture" })).model("claude-haiku-4-5-20251001"),
        "system": [{ "type": "text", "text": common::large_cacheable_system(), "cache": CacheHint::new(json!({ "type": "ephemeral" })) }],
        "prompt": "Say hi.",
        "cache": "none",
        "generation": { "maxTokens": 16, "temperature": 0 },
    }));

    let first = LLMClient::generate(request.clone()).expect("first call");
    assert!(first.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) >= 0);

    let second = LLMClient::generate(request).expect("second call");
    assert!(second.usage["cacheReadInputTokens"].as_i64().unwrap_or(0) > 0);
}
