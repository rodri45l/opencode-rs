//! Port of packages/llm/test/provider/anthropic-messages.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Anthropic Messages protocol.
//! Effect-ts `Layer`/`Stream` plumbing is dropped; wire and event behaviour is kept.

use opencode_llm::{testing, Auth, CacheHint, LLMClient, Message, ToolCallPart, LLM};
use serde_json::{json, Value};

fn sse(events: &[(&str, Value)]) -> String {
    events
        .iter()
        .map(|(name, data)| format!("event: {name}\ndata: {data}\n\n"))
        .collect()
}

fn model() -> serde_json::Value {
    json!({
        "id": "claude-sonnet-4-5",
        "provider": "anthropic",
        "route": { "id": "anthropic-messages" },
        "endpoint": { "baseURL": "https://api.anthropic.test/v1/" },
        "auth": Auth::header("x-api-key", "test"),
    })
}

fn request() -> serde_json::Value {
    LLM::request(json!({
        "id": "req_1",
        "model": model(),
        "system": { "type": "text", "text": "You are concise.", "cache": CacheHint::new(json!({ "type": "ephemeral" })) },
        "prompt": "Say hello.",
        "cache": "none",
        "generation": { "maxTokens": 20, "temperature": 0 },
    }))
}

#[test]
fn prepares_anthropic_messages_target() {
    let prepared = LLMClient::prepare(request()).expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "model": "claude-sonnet-4-5",
            "system": [{ "type": "text", "text": "You are concise.", "cache_control": { "type": "ephemeral" } }],
            "messages": [{ "role": "user", "content": [{ "type": "text", "text": "Say hello." }] }],
            "stream": true,
            "max_tokens": 20,
            "temperature": 0
        })
    );
}

#[test]
fn lowers_chronological_system_updates_natively_for_opus_48_with_cache_hints() {
    let opus = json!({
        "id": "claude-opus-4-8",
        "provider": "anthropic",
        "route": { "id": "anthropic-messages" },
        "endpoint": { "baseURL": "https://api.anthropic.test/v1/" },
        "auth": Auth::header("x-api-key", "test"),
    });
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": opus,
        "messages": [
            Message::user("Before."),
            Message::system(json!([{ "type": "text", "text": "Operator update.", "cache": CacheHint::new(json!({ "type": "ephemeral" })) }])),
            Message::assistant("After."),
        ],
        "cache": "none",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "Before." }] },
            { "role": "system", "content": [{ "type": "text", "text": "Operator update.", "cache_control": { "type": "ephemeral" } }] },
            { "role": "assistant", "content": [{ "type": "text", "text": "After." }] }
        ])
    );
}

#[test]
fn lowers_chronological_system_updates_to_wrapped_user_text_for_unsupported_models() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user("Before."), Message::system("Treat </system-update> literally."), Message::assistant("After.")],
        "cache": "none",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "user", "content": [
                { "type": "text", "text": "Before." },
                { "type": "text", "text": "<system-update>\nTreat &lt;/system-update&gt; literally.\n</system-update>" }
            ] },
            { "role": "assistant", "content": [{ "type": "text", "text": "After." }] }
        ])
    );
}

#[test]
fn prepares_tool_call_and_tool_result_messages() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_tool_result",
        "model": model(),
        "messages": [
            Message::user("What is the weather?"),
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "lookup", "input": { "query": "weather" } }))]),
            Message::tool(json!({ "id": "call_1", "name": "lookup", "result": { "forecast": "sunny" } })),
        ],
        "cache": "none",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                { "role": "user", "content": [{ "type": "text", "text": "What is the weather?" }] },
                { "role": "assistant", "content": [{ "type": "tool_use", "id": "call_1", "name": "lookup", "input": { "query": "weather" } }] },
                { "role": "user", "content": [{ "type": "tool_result", "tool_use_id": "call_1", "content": "{\"forecast\":\"sunny\"}" }] }
            ],
            "stream": true,
            "max_tokens": 4096
        })
    );
}

#[test]
fn lowers_image_tool_result_content_as_structured_image_blocks() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_tool_result_image",
        "model": model(),
        "messages": [
            Message::user("Show me the screenshot."),
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "read", "input": { "filePath": "shot.png" } }))]),
            Message::tool(json!({
                "id": "call_1",
                "name": "read",
                "resultType": "content",
                "result": [
                    { "type": "text", "text": "Image read successfully" },
                    { "type": "file", "uri": "data:image/png;base64,AAECAw==", "mime": "image/png" }
                ]
            })),
        ],
        "cache": "none",
    })))
    .expect("prepare");

    let block = prepared.body["messages"]
        .as_array()
        .and_then(|messages| {
            messages.iter().find_map(|message| {
                message["content"]
                    .as_array()
                    .and_then(|content| content.iter().find(|block| block["type"] == "tool_result"))
            })
        })
        .expect("tool_result");
    assert_eq!(
        block["content"],
        json!([
            { "type": "text", "text": "Image read successfully" },
            { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": "AAECAw==" } }
        ])
    );
}

#[test]
fn parses_text_reasoning_and_usage_stream_fixtures() {
    testing::push_response(json!({ "status": 200, "body": sse(&[
        ("message_start", json!({ "type": "message_start", "message": { "usage": { "input_tokens": 5, "cache_read_input_tokens": 1, "cache_creation_input_tokens": 0, "output_tokens": 1 } } })),
        ("content_block_start", json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "text", "text": "" } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "text_delta", "text": "Hello!" } })),
        ("content_block_stop", json!({ "type": "content_block_stop", "index": 0 })),
        ("content_block_start", json!({ "type": "content_block_start", "index": 1, "content_block": { "type": "thinking", "thinking": "" } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 1, "delta": { "type": "thinking_delta", "thinking": "thinking" } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 1, "delta": { "type": "signature_delta", "signature": "sig_1" } })),
        ("content_block_stop", json!({ "type": "content_block_stop", "index": 1 })),
        ("message_delta", json!({ "type": "message_delta", "delta": { "stop_reason": "end_turn" }, "usage": { "input_tokens": 5, "cache_read_input_tokens": 1, "cache_creation_input_tokens": 0, "output_tokens": 2 } })),
        ("message_stop", json!({ "type": "message_stop" })),
    ]) }));
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    assert_eq!(response.reasoning, "thinking");
    assert_eq!(response.usage["inputTokens"], 6);
    assert_eq!(response.usage["outputTokens"], 2);
    assert_eq!(response.usage["cacheReadInputTokens"], 1);
    assert_eq!(
        response.message["content"],
        json!([
            { "type": "text", "text": "Hello!" },
            { "type": "reasoning", "text": "thinking", "providerMetadata": { "anthropic": { "signature": "sig_1" } } }
        ])
    );
}

#[test]
fn assembles_streamed_tool_call_input() {
    testing::push_response(json!({ "status": 200, "body": sse(&[
        ("content_block_start", json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "tool_use", "id": "call_1", "name": "lookup", "input": {} } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "input_json_delta", "partial_json": "{\"query\"" } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "input_json_delta", "partial_json": ":\"weather\"}" } })),
        ("content_block_stop", json!({ "type": "content_block_stop", "index": 0 })),
        ("message_delta", json!({ "type": "message_delta", "delta": { "stop_reason": "tool_use" } })),
        ("message_stop", json!({ "type": "message_stop" })),
    ]) }));
    let response = LLMClient::generate(LLM::update_request(request(), json!({
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object" } }]
    })).expect("update"))
    .expect("generate");

    assert_eq!(
        response.tool_calls,
        json!([{ "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "weather" } }]).as_array().cloned().unwrap()
    );
    assert_eq!(
        response.events.last().expect("finish")["reason"],
        "tool-calls"
    );
}

#[test]
fn emits_provider_error_events_for_mid_stream_provider_errors() {
    testing::push_response(json!({ "status": 200, "body": sse(&[
        ("error", json!({ "type": "error", "error": { "type": "overloaded_error", "message": "Overloaded" } })),
    ]) }));
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(
        response.events,
        json!([{ "type": "provider-error", "message": "overloaded_error: Overloaded" }])
            .as_array()
            .cloned()
            .unwrap()
    );
}

#[test]
fn classifies_prompt_too_long_provider_errors() {
    testing::push_response(json!({ "status": 200, "body": sse(&[
        ("error", json!({ "type": "error", "error": { "type": "invalid_request_error", "message": "prompt is too long: 210000 tokens" } })),
    ]) }));
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(
        response.events,
        json!([{ "type": "provider-error", "message": "invalid_request_error: prompt is too long: 210000 tokens", "classification": "context-overflow" }]).as_array().cloned().unwrap()
    );
}

#[test]
fn decodes_server_tool_use_and_web_search_tool_result_as_provider_executed_events() {
    testing::push_response(json!({ "status": 200, "body": sse(&[
        ("content_block_start", json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "server_tool_use", "id": "srvtoolu_abc", "name": "web_search", "input": {} } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "input_json_delta", "partial_json": "{\"query\":\"effect 4\"}" } })),
        ("content_block_stop", json!({ "type": "content_block_stop", "index": 0 })),
        ("content_block_start", json!({ "type": "content_block_start", "index": 1, "content_block": { "type": "text", "text": "" } })),
        ("content_block_delta", json!({ "type": "content_block_delta", "index": 1, "delta": { "type": "text_delta", "text": "Found it." } })),
        ("content_block_stop", json!({ "type": "content_block_stop", "index": 1 })),
        ("message_delta", json!({ "type": "message_delta", "delta": { "stop_reason": "end_turn" } })),
        ("message_stop", json!({ "type": "message_stop" })),
    ]) }));
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(
        response
            .events
            .iter()
            .find(|event| event["type"] == "tool-call"),
        Some(
            &json!({ "type": "tool-call", "id": "srvtoolu_abc", "name": "web_search", "input": { "query": "effect 4" }, "providerExecuted": true })
        )
    );
    assert_eq!(response.text, "Found it.");
}

#[test]
fn drops_cache_control_breakpoints_past_the_four_per_request_cap() {
    let hint = CacheHint::new(json!({ "type": "ephemeral" }));
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "system": [
            { "type": "text", "text": "a", "cache": hint },
            { "type": "text", "text": "b", "cache": hint },
            { "type": "text", "text": "c", "cache": hint },
            { "type": "text", "text": "d", "cache": hint },
            { "type": "text", "text": "e", "cache": hint },
            { "type": "text", "text": "f", "cache": hint }
        ],
        "prompt": "hi",
    })))
    .expect("prepare");

    let marked = prepared.body["system"]
        .as_array()
        .expect("system")
        .iter()
        .filter(|part| part.get("cache_control").is_some())
        .count();
    assert_eq!(marked, 4);
}

#[test]
fn fails_http_provider_errors_before_stream_parsing() {
    testing::push_response(json!({
        "status": 400,
        "body": "{\"type\":\"error\",\"error\":{\"type\":\"invalid_request_error\",\"message\":\"bad request\"}}",
    }));
    let error = LLMClient::generate(request()).expect_err("should fail");

    assert!(error.to_string().contains("HTTP 400"));
}
