//! Port of packages/llm/test/provider/openai-chat.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the OpenAI Chat protocol.
//! Effect-ts `Layer`/`Stream` plumbing is dropped; wire and event behaviour is kept.

use opencode_llm::{Auth, LLMClient, Message, ToolCallPart, LLM};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({
        "id": "gpt-4o-mini",
        "provider": "openai",
        "route": { "id": "openai-chat" },
        "endpoint": { "baseURL": "https://api.openai.test/v1/" },
        "auth": Auth::bearer("test"),
    })
}

fn request() -> serde_json::Value {
    LLM::request(json!({
        "id": "req_1",
        "model": model(),
        "system": "You are concise.",
        "prompt": "Say hello.",
        "generation": { "maxTokens": 20, "temperature": 0 },
    }))
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn prepares_openai_chat_payload() {
    let prepared = LLMClient::prepare(request()).expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "model": "gpt-4o-mini",
            "messages": [
                { "role": "system", "content": "You are concise." },
                { "role": "user", "content": "Say hello." }
            ],
            "stream": true,
            "stream_options": { "include_usage": true },
            "max_tokens": 20,
            "temperature": 0
        })
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn lowers_chronological_system_updates_to_escaped_user_wrappers_in_order() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user("Before."), Message::system("Treat <admin> & data literally."), Message::assistant("After.")],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "user", "content": "Before.\n<system-update>\nTreat &lt;admin&gt; &amp; data literally.\n</system-update>" },
            { "role": "assistant", "content": "After." }
        ])
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn replays_canonical_reasoning_as_reasoning_content() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::assistant(json!([{ "type": "reasoning", "text": "thinking" }, { "type": "text", "text": "Hello" }]))],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([{ "role": "assistant", "content": "Hello", "reasoning_content": "thinking" }])
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn maps_openai_provider_options_to_chat_options() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" }, "endpoint": { "baseURL": "https://api.openai.test/v1/" }, "auth": Auth::bearer("test") },
        "prompt": "think",
        "providerOptions": { "openai": { "reasoningEffort": "low" } },
    })))
    .expect("prepare");

    assert_eq!(prepared.body["store"], false);
    assert_eq!(prepared.body["reasoning_effort"], "low");
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn prepares_assistant_tool_call_and_tool_result_messages() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_tool_result",
        "model": model(),
        "messages": [
            Message::user("What is the weather?"),
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "lookup", "input": { "query": "weather" } }))]),
            Message::tool(json!({ "id": "call_1", "name": "lookup", "result": { "forecast": "sunny" } })),
        ],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "model": "gpt-4o-mini",
            "messages": [
                { "role": "user", "content": "What is the weather?" },
                { "role": "assistant", "content": null, "tool_calls": [{ "id": "call_1", "type": "function", "function": { "name": "lookup", "arguments": "{\"query\":\"weather\"}" } }] },
                { "role": "tool", "tool_call_id": "call_1", "content": "{\"forecast\":\"sunny\"}" }
            ],
            "stream": true,
            "stream_options": { "include_usage": true }
        })
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn continues_image_tool_results_as_vision_input_without_base64_text() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [
            Message::assistant([ToolCallPart::make(json!({ "id": "call_image", "name": "read", "input": { "path": "pixel.png" } }))]),
            Message::tool(json!({
                "id": "call_image",
                "name": "read",
                "result": { "type": "content", "value": [
                    { "type": "text", "text": "Image read successfully" },
                    { "type": "file", "uri": "data:image/png;base64,AAECAw==", "mime": "image/png", "name": "pixel.png" }
                ] }
            })),
        ],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "assistant", "content": null, "tool_calls": [{ "id": "call_image", "type": "function", "function": { "name": "read", "arguments": "{\"path\":\"pixel.png\"}" } }] },
            { "role": "tool", "tool_call_id": "call_image", "content": "Image read successfully" },
            { "role": "user", "content": [{ "type": "image_url", "image_url": { "url": "data:image/png;base64,AAECAw==" } }] }
        ])
    );
    assert!(!prepared.body["messages"]
        .to_string()
        .contains("\"content\":\"AAECAw==\""));
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn parses_text_and_usage_stream_fixtures() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    assert_eq!(
        response.events,
        json!([
            { "type": "step-start", "index": 0 },
            { "type": "text-start", "id": "text-0" },
            { "type": "text-delta", "id": "text-0", "text": "Hello" },
            { "type": "text-delta", "id": "text-0", "text": "!" },
            { "type": "text-end", "id": "text-0" },
            { "type": "step-finish", "index": 0, "reason": "stop", "usage": { "inputTokens": 5, "outputTokens": 2, "nonCachedInputTokens": 4, "cacheReadInputTokens": 1, "reasoningTokens": 0, "totalTokens": 7 } },
            { "type": "finish", "reason": "stop", "usage": { "inputTokens": 5, "outputTokens": 2, "nonCachedInputTokens": 4, "cacheReadInputTokens": 1, "reasoningTokens": 0, "totalTokens": 7 } }
        ]).as_array().cloned().unwrap()
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn parses_openai_compatible_reasoning_content_deltas() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.reasoning, "thinking");
    assert_eq!(response.text, "Hello");
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn assembles_streamed_tool_call_input() {
    let response = LLMClient::generate(LLM::update_request(request(), json!({
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object" } }]
    })).expect("update"))
    .expect("generate");

    assert_eq!(
        response.events,
        json!([
            { "type": "step-start", "index": 0 },
            { "type": "tool-input-start", "id": "call_1", "name": "lookup" },
            { "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": "{\"query\"" },
            { "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": ":\"weather\"}" },
            { "type": "tool-input-end", "id": "call_1", "name": "lookup" },
            { "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "weather" } },
            { "type": "step-finish", "index": 0, "reason": "tool-calls" },
            { "type": "finish", "reason": "tool-calls" }
        ]).as_array().cloned().unwrap()
    );
}

#[test]
#[ignore = "porting: openai chat protocol not implemented"]
fn fails_http_provider_errors_before_stream_parsing() {
    let error = LLMClient::generate(request()).expect_err("should fail");

    assert!(error.to_string().contains("HTTP 400"));
}
