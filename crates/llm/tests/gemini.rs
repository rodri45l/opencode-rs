//! Port of packages/llm/test/provider/gemini.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Gemini protocol.
//! Effect-ts `Layer`/`Stream` plumbing is dropped; wire and event behaviour is kept.

use opencode_llm::{Auth, LLMClient, Message, ProviderShared, ToolCallPart, LLM};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({
        "id": "gemini-2.5-flash",
        "provider": "google",
        "route": { "id": "gemini" },
        "endpoint": { "baseURL": "https://generativelanguage.test/v1beta/" },
        "auth": Auth::header("x-goog-api-key", "test"),
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
#[ignore = "porting: gemini protocol not implemented"]
fn prepares_gemini_target() {
    let prepared = LLMClient::prepare(request()).expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "contents": [{ "role": "user", "parts": [{ "text": "Say hello." }] }],
            "systemInstruction": { "parts": [{ "text": "You are concise." }] },
            "generationConfig": { "maxOutputTokens": 20, "temperature": 0 }
        })
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn lowers_chronological_system_updates_to_wrapped_user_text_in_order() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user("Before."), Message::system("Update."), Message::assistant("After.")],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["contents"],
        json!([
            { "role": "user", "parts": [{ "text": "Before." }, { "text": "<system-update>\nUpdate.\n</system-update>" }] },
            { "role": "model", "parts": [{ "text": "After." }] }
        ])
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn prepares_multimodal_user_input_and_tool_history() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_tool_result",
        "model": model(),
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object", "properties": { "query": { "type": "string" } } } }],
        "toolChoice": { "type": "tool", "name": "lookup" },
        "messages": [
            Message::user(json!([{ "type": "text", "text": "What is in this image?" }, { "type": "media", "mediaType": "image/png", "data": "AAECAw==" }])),
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "lookup", "input": { "query": "weather" } }))]),
            Message::tool(json!({ "id": "call_1", "name": "lookup", "result": { "forecast": "sunny" } })),
        ],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "contents": [
                { "role": "user", "parts": [{ "text": "What is in this image?" }, { "inlineData": { "mimeType": "image/png", "data": "AAECAw==" } }] },
                { "role": "model", "parts": [{ "functionCall": { "name": "lookup", "args": { "query": "weather" } } }] },
                { "role": "user", "parts": [{ "functionResponse": { "name": "lookup", "response": { "name": "lookup", "content": "{\"forecast\":\"sunny\"}" } } }] }
            ],
            "tools": [{ "functionDeclarations": [{ "name": "lookup", "description": "Lookup data", "parameters": { "type": "object", "properties": { "query": { "type": "string" } } } }] }],
            "toolConfig": { "functionCallingConfig": { "mode": "ANY", "allowedFunctionNames": ["lookup"] } }
        })
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn continues_image_tool_results_as_inline_vision_input_without_base64_text() {
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
        prepared.body["contents"],
        json!([
            { "role": "model", "parts": [{ "functionCall": { "name": "read", "args": { "path": "pixel.png" } } }] },
            { "role": "user", "parts": [
                { "functionResponse": { "name": "read", "response": { "name": "read", "content": "Image read successfully" } } },
                { "inlineData": { "mimeType": "image/png", "data": "AAECAw==" } }
            ] }
        ])
    );
    assert!(!prepared.body["contents"]
        .to_string()
        .contains("\"content\":\"AAECAw==\""));
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn omits_tools_when_tool_choice_is_none() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_no_tools",
        "model": model(),
        "prompt": "Say hello.",
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object" } }],
        "toolChoice": { "type": "none" },
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body,
        json!({ "contents": [{ "role": "user", "parts": [{ "text": "Say hello." }] }] })
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn sanitizes_integer_enums_dangling_required_and_untyped_arrays() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_schema_patch",
        "model": model(),
        "prompt": "Use the tool.",
        "tools": [{
            "name": "lookup",
            "description": "Lookup data",
            "inputSchema": {
                "type": "object",
                "required": ["status", "missing"],
                "properties": {
                    "status": { "type": "integer", "enum": [1, 2] },
                    "tags": { "type": "array" },
                    "name": { "type": "string", "properties": { "ignored": { "type": "string" } }, "required": ["ignored"] }
                }
            }
        }],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["tools"][0]["functionDeclarations"][0]["parameters"],
        json!({
            "type": "object",
            "required": ["status"],
            "properties": {
                "status": { "type": "string", "enum": ["1", "2"] },
                "tags": { "type": "array", "items": { "type": "string" } },
                "name": { "type": "string" }
            }
        })
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn parses_text_reasoning_and_usage_stream_fixtures() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    assert_eq!(response.reasoning, "thinking");
    assert_eq!(response.usage["inputTokens"], 5);
    assert_eq!(response.usage["outputTokens"], 3);
    assert_eq!(response.usage["reasoningTokens"], 1);
    assert_eq!(response.usage["cacheReadInputTokens"], 1);
    assert_eq!(response.usage["totalTokens"], 7);
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn preserves_thought_signature_for_reasoning_and_tool_call_continuation() {
    let response = LLMClient::generate(request()).expect("generate");

    let reasoning_end = response
        .events
        .iter()
        .find(|event| event["type"] == "reasoning-end")
        .expect("reasoning-end");
    assert_eq!(
        reasoning_end["providerMetadata"],
        json!({ "google": { "thoughtSignature": "thought_sig" } })
    );
    let tool_call = response
        .events
        .iter()
        .find(|event| event["type"] == "tool-call")
        .expect("tool-call");
    assert_eq!(
        tool_call["providerMetadata"],
        json!({ "google": { "thoughtSignature": "tool_sig" } })
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn assigns_unique_ids_to_multiple_streamed_tool_calls() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(
        response.tool_calls,
        json!([
            { "type": "tool-call", "id": "tool_0", "name": "lookup", "input": { "query": "weather" } },
            { "type": "tool-call", "id": "tool_1", "name": "lookup", "input": { "query": "news" } }
        ]).as_array().cloned().unwrap()
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn maps_length_and_content_filter_finish_reasons() {
    let length = LLMClient::generate(request()).expect("generate");
    let filtered = LLMClient::generate(request()).expect("generate");

    assert_eq!(length.events.last().expect("finish")["reason"], "length");
    assert_eq!(
        filtered.events.last().expect("finish")["reason"],
        "content-filter"
    );
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn rejects_unsupported_assistant_media_content() {
    let error = LLMClient::prepare(LLM::request(json!({
        "id": "req_media",
        "model": model(),
        "messages": [Message::assistant(json!({ "type": "media", "mediaType": "image/png", "data": "AAECAw==" }))],
    })))
    .expect_err("should fail");

    assert!(error.to_string().contains(
        "Gemini assistant messages only support text, reasoning, and tool-call content for now"
    ));
}

#[test]
#[ignore = "porting: gemini protocol not implemented"]
fn rejects_oversized_image_input() {
    let oversized = "A".repeat(ProviderShared::MAX_MEDIA_ENCODED_BYTES + 4);
    let error = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user(json!({ "type": "media", "mediaType": "image/png", "data": oversized }))],
    })))
    .expect_err("should fail");

    assert!(error.to_string().contains("encoded limit"));
}
