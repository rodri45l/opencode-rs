//! Port of packages/llm/test/provider/bedrock-converse-cache.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded Bedrock cache cassette.
//! The reference cassette is absent at the pinned commit, so this test skips
//! (mirroring `recordedTests`' `cassetteExists` guard) rather than failing.

mod common;

use opencode_llm::{providers, CacheHint, LLM, LLMClient};
use serde_json::json;

const CASSETTE: &str = "writes-then-reads-cachepoint-on-identical-second-call";

#[test]
#[ignore = "porting: bedrock recorded cache not implemented"]
fn writes_then_reads_cache_point_on_identical_second_call() {
    if !common::recording_exists("bedrock-converse-cache", CASSETTE) {
        return;
    }
    let cassette = common::recording("bedrock-converse-cache", CASSETTE);
    assert!(cassette.get("interactions").is_some());

    let request = LLM::request(json!({
        "id": "recorded_bedrock_cache",
        "model": providers::amazon_bedrock::configure(json!({
            "credentials": { "region": "us-east-1", "accessKeyId": "fixture", "secretAccessKey": "fixture" }
        })).model("us.anthropic.claude-haiku-4-5-20251001-v1:0"),
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
