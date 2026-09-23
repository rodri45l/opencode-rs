//! Port of packages/llm/test/provider/anthropic-messages.recorded.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the recorded Anthropic sad-path cassette.

mod common;

use opencode_llm::{providers, LLMClient, Message, ToolCallPart, LLM};
use serde_json::json;

#[test]
#[ignore = "porting: anthropic recorded sad path not implemented"]
fn rejects_malformed_assistant_tool_order() {
    let cassette = common::recording(
        "anthropic-messages",
        "rejects-malformed-assistant-tool-order-without-patch",
    );
    assert!(
        cassette.get("interactions").is_some(),
        "cassette carries the recorded 400"
    );

    let request = LLM::request(json!({
        "id": "recorded_anthropic_malformed_tool_order",
        "model": providers::anthropic::configure(json!({ "apiKey": "fixture" })).model("claude-haiku-4-5-20251001"),
        "messages": [
            Message::assistant(json!([
                ToolCallPart::make(json!({ "id": "call_1", "name": common::WEATHER_TOOL_NAME, "input": { "city": "Paris" } })),
                { "type": "text", "text": "I will check the weather." }
            ])),
            Message::tool(json!({ "id": "call_1", "name": common::WEATHER_TOOL_NAME, "result": { "temperature": "72F" } })),
            Message::user("Use that result to answer briefly."),
        ],
        "tools": [{ "name": common::WEATHER_TOOL_NAME, "description": "Get weather", "inputSchema": { "type": "object", "properties": {} } }],
    }));

    let error = LLMClient::generate(request).expect_err("should fail");

    assert!(error.to_string().contains("HTTP 400"));
}
