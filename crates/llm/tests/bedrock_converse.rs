//! Port of packages/llm/test/provider/bedrock-converse.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Bedrock Converse protocol.
//! The AWS binary event-stream framing is represented as decoded frames; the
//! assertions pin the normalised events and wire shapes.

use opencode_llm::{Auth, CacheHint, LLMClient, Message, ToolCallPart, LLM};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({
        "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
        "provider": "amazon-bedrock",
        "route": { "id": "bedrock-converse" },
        "endpoint": { "baseURL": "https://bedrock-runtime.test" },
        "auth": Auth::bearer("test-bearer"),
    })
}

fn base_request() -> serde_json::Value {
    LLM::request(json!({
        "id": "req_1",
        "model": model(),
        "system": "You are concise.",
        "prompt": "Say hello.",
        "cache": "none",
        "generation": { "maxTokens": 64, "temperature": 0 },
    }))
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn prepares_converse_target_with_system_inference_config_and_messages() {
    let prepared = LLMClient::prepare(base_request()).expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "modelId": "anthropic.claude-3-5-sonnet-20240620-v1:0",
            "system": [{ "text": "You are concise." }],
            "messages": [{ "role": "user", "content": [{ "text": "Say hello." }] }],
            "inferenceConfig": { "maxTokens": 64, "temperature": 0 }
        })
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn passes_top_k_through_additional_model_request_fields() {
    let prepared = LLMClient::prepare(
        LLM::update_request(
            base_request(),
            json!({
                "generation": { "maxTokens": 64, "temperature": 0, "topK": 40 }
            }),
        )
        .expect("update"),
    )
    .expect("prepare");

    assert_eq!(
        prepared.body["inferenceConfig"],
        json!({ "maxTokens": 64, "temperature": 0 })
    );
    assert_eq!(
        prepared.body["additionalModelRequestFields"],
        json!({ "top_k": 40 })
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn prepares_tool_config_with_tool_spec_and_tool_choice() {
    let prepared = LLMClient::prepare(LLM::update_request(base_request(), json!({
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] } }],
        "toolChoice": { "type": "required" }
    })).expect("update"))
    .expect("prepare");

    assert_eq!(
        prepared.body["toolConfig"],
        json!({
            "tools": [{ "toolSpec": {
                "name": "lookup",
                "description": "Lookup data",
                "inputSchema": { "json": { "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] } }
            } }],
            "toolChoice": { "any": {} }
        })
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn lowers_assistant_tool_call_and_tool_result_history() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_history",
        "model": model(),
        "messages": [
            Message::user("What is the weather?"),
            Message::assistant([ToolCallPart::make(json!({ "id": "tool_1", "name": "lookup", "input": { "query": "weather" } }))]),
            Message::tool(json!({ "id": "tool_1", "name": "lookup", "result": { "forecast": "sunny" } })),
        ],
        "cache": "none",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "user", "content": [{ "text": "What is the weather?" }] },
            { "role": "assistant", "content": [{ "toolUse": { "toolUseId": "tool_1", "name": "lookup", "input": { "query": "weather" } } }] },
            { "role": "user", "content": [{ "toolResult": { "toolUseId": "tool_1", "content": [{ "json": { "forecast": "sunny" } }], "status": "success" } }] }
        ])
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn decodes_text_delta_message_stop_and_metadata_usage() {
    let response = LLMClient::generate(base_request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    let finishes: Vec<&serde_json::Value> = response
        .events
        .iter()
        .filter(|event| event["type"] == "finish")
        .collect();
    assert_eq!(finishes.len(), 1);
    assert_eq!(finishes[0]["reason"], "stop");
    assert_eq!(response.usage["inputTokens"], 5);
    assert_eq!(response.usage["outputTokens"], 2);
    assert_eq!(response.usage["totalTokens"], 7);
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn assembles_streamed_tool_call_input() {
    let response = LLMClient::generate(LLM::update_request(base_request(), json!({
        "tools": [{ "name": "lookup", "description": "Lookup", "inputSchema": { "type": "object" } }]
    })).expect("update"))
    .expect("generate");

    assert_eq!(response.tool_calls, json!([{ "type": "tool-call", "id": "tool_1", "name": "lookup", "input": { "query": "weather" } }]).as_array().cloned().unwrap());
    let deltas: Vec<&serde_json::Value> = response
        .events
        .iter()
        .filter(|event| event["type"] == "tool-input-delta")
        .collect();
    assert_eq!(
        deltas,
        vec![
            &json!({ "type": "tool-input-delta", "id": "tool_1", "name": "lookup", "text": "{\"query\"" }),
            &json!({ "type": "tool-input-delta", "id": "tool_1", "name": "lookup", "text": ":\"weather\"}" })
        ]
    );
    assert_eq!(
        response.events.last().expect("finish")["reason"],
        "tool-calls"
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn emits_provider_error_for_throttling_exception() {
    let response = LLMClient::generate(base_request()).expect("generate");

    assert_eq!(
        response
            .events
            .iter()
            .find(|event| event["type"] == "provider-error"),
        Some(&json!({ "type": "provider-error", "message": "Slow down", "retryable": true }))
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn classifies_input_too_long_validation_exceptions() {
    let response = LLMClient::generate(base_request()).expect("generate");

    assert_eq!(
        response
            .events
            .iter()
            .find(|event| event["type"] == "provider-error"),
        Some(
            &json!({ "type": "provider-error", "message": "Input is too long for requested model", "classification": "context-overflow", "retryable": false })
        )
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn rejects_requests_with_no_auth_path() {
    let unsigned = json!({
        "id": "anthropic.claude-3-5-sonnet-20240620-v1:0",
        "provider": "amazon-bedrock",
        "route": { "id": "bedrock-converse" },
        "endpoint": { "baseURL": "https://bedrock-runtime.test" },
    });
    let error = LLMClient::generate(
        LLM::update_request(base_request(), json!({ "model": unsigned })).expect("update"),
    )
    .expect_err("should fail");

    assert!(error
        .to_string()
        .contains("Bedrock Converse requires either route bearer auth or AWS credentials"));
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn emits_cache_point_markers_after_system_user_and_assistant_text() {
    let cache = CacheHint::new(json!({ "type": "ephemeral" }));
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_cache",
        "model": model(),
        "system": [{ "type": "text", "text": "System prefix.", "cache": cache }],
        "messages": [
            Message::user(json!([{ "type": "text", "text": "User prefix.", "cache": cache }])),
            Message::assistant(json!([{ "type": "text", "text": "Assistant prefix.", "cache": cache }])),
        ],
        "generation": { "maxTokens": 16, "temperature": 0 },
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["system"],
        json!([{ "text": "System prefix." }, { "cachePoint": { "type": "default" } }])
    );
    assert_eq!(
        prepared.body["messages"][0]["content"],
        json!([{ "text": "User prefix." }, { "cachePoint": { "type": "default" } }])
    );
    assert_eq!(
        prepared.body["messages"][1]["content"],
        json!([{ "text": "Assistant prefix." }, { "cachePoint": { "type": "default" } }])
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn lowers_image_media_into_bedrock_image_blocks() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_image",
        "model": model(),
        "messages": [Message::user(json!([
            { "type": "text", "text": "What is in this image?" },
            { "type": "media", "mediaType": "image/png", "data": "AAAA" },
            { "type": "media", "mediaType": "image/jpeg", "data": "BBBB" },
            { "type": "media", "mediaType": "image/jpg", "data": "CCCC" },
            { "type": "media", "mediaType": "image/webp", "data": "DDDD" }
        ]))],
        "cache": "none",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"][0]["content"],
        json!([
            { "text": "What is in this image?" },
            { "image": { "format": "png", "source": { "bytes": "AAAA" } } },
            { "image": { "format": "jpeg", "source": { "bytes": "BBBB" } } },
            { "image": { "format": "jpeg", "source": { "bytes": "CCCC" } } },
            { "image": { "format": "webp", "source": { "bytes": "DDDD" } } }
        ])
    );
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn rejects_unsupported_image_and_document_media_types() {
    let bad_image = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user(json!([{ "type": "media", "mediaType": "image/svg+xml", "data": "x" }]))],
    })))
    .expect_err("image");
    assert!(bad_image
        .to_string()
        .contains("Bedrock Converse does not support image media type image/svg+xml"));

    let bad_doc = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user(json!([{ "type": "media", "mediaType": "application/x-tar", "data": "x", "filename": "a.tar" }]))],
    })))
    .expect_err("document");
    assert!(bad_doc
        .to_string()
        .contains("Bedrock Converse does not support media type application/x-tar"));
}

#[test]
#[ignore = "porting: bedrock converse protocol not implemented"]
fn drops_cache_point_markers_past_the_four_per_request_cap() {
    let cache = CacheHint::new(json!({ "type": "ephemeral" }));
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "system": [
            { "type": "text", "text": "a", "cache": cache },
            { "type": "text", "text": "b", "cache": cache },
            { "type": "text", "text": "c", "cache": cache },
            { "type": "text", "text": "d", "cache": cache },
            { "type": "text", "text": "e", "cache": cache },
            { "type": "text", "text": "f", "cache": cache }
        ],
        "prompt": "hi",
    })))
    .expect("prepare");

    let marked = prepared.body["system"]
        .as_array()
        .expect("system")
        .iter()
        .filter(|part| part.get("cachePoint").is_some())
        .count();
    assert_eq!(marked, 4);
}
