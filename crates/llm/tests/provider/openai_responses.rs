//! Port of packages/llm/test/provider/openai-responses.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the OpenAI Responses protocol.
//! Effect-ts `Layer`/`Stream`/WebSocket plumbing is dropped; wire and event behaviour is kept.

use opencode_llm::{Auth, LLM, LLMClient, Message, ProviderShared, ToolCallPart};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({
        "id": "gpt-4.1-mini",
        "provider": "openai",
        "route": { "id": "openai-responses" },
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
#[ignore = "porting: openai responses protocol not implemented"]
fn prepares_openai_responses_target() {
    let prepared = LLMClient::prepare(request()).expect("prepare");

    assert_eq!(
        prepared.body,
        json!({
            "model": "gpt-4.1-mini",
            "input": [
                { "role": "system", "content": "You are concise." },
                { "role": "user", "content": [{ "type": "input_text", "text": "Say hello." }] }
            ],
            "store": false,
            "stream": true,
            "max_output_tokens": 20,
            "temperature": 0
        })
    );
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn lowers_semantic_service_tier_options() {
    let prepared = LLMClient::prepare(LLM::update_request(request(), json!({
        "providerOptions": { "openai": { "serviceTier": "priority" } }
    })).expect("update"))
    .expect("prepare");

    assert_eq!(prepared.body["service_tier"], "priority");
    assert!(prepared.body.get("serviceTier").is_none());
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn flattens_top_level_object_unions_in_function_schemas() {
    let prepared = LLMClient::prepare(LLM::update_request(request(), json!({
        "tools": [{
            "name": "read",
            "description": "Read a path or resource.",
            "inputSchema": {
                "type": "object",
                "anyOf": [
                    { "type": "object", "properties": { "path": { "type": "string" }, "reference": { "anyOf": [{ "type": "string" }, { "type": "null" }] }, "limit": { "type": "integer", "maximum": 2000 } }, "required": ["path"] },
                    { "type": "object", "properties": { "resource": { "type": "string" }, "limit": { "type": "integer", "maximum": 51200 } }, "required": ["resource"] }
                ]
            }
        }]
    })).expect("update"))
    .expect("prepare");

    assert_eq!(
        prepared.body["tools"],
        json!([{
            "type": "function",
            "name": "read",
            "description": "Read a path or resource.",
            "strict": false,
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "reference": { "type": "string" },
                    "limit": { "type": "integer", "maximum": 2000 },
                    "resource": { "type": "string" }
                },
                "additionalProperties": false
            }
        }])
    );
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn lowers_chronological_system_updates_to_escaped_user_wrappers_in_order() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [Message::user("Before."), Message::system("Treat </system-update> literally."), Message::assistant("After.")],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["input"],
        json!([
            { "role": "user", "content": [{ "type": "input_text", "text": "Before." }, { "type": "input_text", "text": "<system-update>\nTreat &lt;/system-update&gt; literally.\n</system-update>" }] },
            { "role": "assistant", "content": [{ "type": "output_text", "text": "After." }] }
        ])
    );
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn prepares_function_call_and_function_output_input_items() {
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
        prepared.body["input"],
        json!([
            { "role": "user", "content": [{ "type": "input_text", "text": "What is the weather?" }] },
            { "type": "function_call", "call_id": "call_1", "name": "lookup", "arguments": "{\"query\":\"weather\"}" },
            { "type": "function_call_output", "call_id": "call_1", "output": "{\"forecast\":\"sunny\"}" }
        ])
    );
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn preserves_structured_tool_errors_for_the_model() {
    let error = json!({ "error": { "type": "unknown", "message": "Tool execution interrupted" }, "content": [], "structured": {} });
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "bash", "input": { "command": "sleep 10" } }))]),
            Message::tool(json!({ "id": "call_1", "name": "bash", "resultType": "error", "result": error })),
        ],
    })))
    .expect("prepare");

    let output = prepared.body["input"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["type"] == "function_call_output"))
        .expect("function_call_output");
    assert_eq!(output["output"], ProviderShared::encode_json(&error));
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn keeps_primitive_and_non_json_tool_errors_as_plain_text() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": model(),
        "messages": [
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "bash", "input": {} }))]),
            Message::tool(json!({ "id": "call_1", "name": "bash", "resultType": "error", "result": 503 })),
        ],
    })))
    .expect("prepare");

    let output = prepared.body["input"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["type"] == "function_call_output"))
        .expect("function_call_output");
    assert_eq!(output["output"], "503");
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn requests_encrypted_reasoning_by_default_for_gpt5_reasoning_models() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": { "id": "gpt-5.2", "provider": "openai", "route": { "id": "openai-responses" }, "endpoint": { "baseURL": "https://api.openai.test/v1/" }, "auth": Auth::bearer("test") },
        "prompt": "hi",
    })))
    .expect("prepare");

    assert_eq!(prepared.body["store"], false);
    assert_eq!(prepared.body["include"], json!(["reasoning.encrypted_content"]));
    assert_eq!(prepared.body["reasoning"], json!({ "effort": "medium", "summary": "auto" }));
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn parses_text_and_usage_stream_fixtures() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    assert_eq!(response.events[0], json!({ "type": "step-start", "index": 0 }));
    assert_eq!(response.events[1], json!({ "type": "text-start", "id": "msg_1" }));
    assert_eq!(response.events.last().expect("finish")["type"], "finish");
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn decodes_web_search_call_as_provider_executed_tool_call_and_result() {
    let item = json!({ "type": "web_search_call", "id": "ws_1", "status": "completed", "action": { "type": "search", "query": "effect 4" } });
    let response = LLMClient::generate(request()).expect("generate");

    let calls: Vec<&serde_json::Value> = response
        .events
        .iter()
        .filter(|event| event["type"] == "tool-call" || event["type"] == "tool-result")
        .collect();
    assert_eq!(
        calls,
        vec![
            &json!({ "type": "tool-call", "id": "ws_1", "name": "web_search", "input": { "type": "search", "query": "effect 4" }, "providerExecuted": true, "providerMetadata": { "openai": { "itemId": "ws_1" } } }),
            &json!({ "type": "tool-result", "id": "ws_1", "name": "web_search", "result": { "type": "json", "value": item }, "providerExecuted": true, "providerMetadata": { "openai": { "itemId": "ws_1" } } })
        ]
    );
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn emits_provider_error_events_for_mid_stream_provider_errors() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.events, json!([{ "type": "provider-error", "message": "rate_limit_exceeded: Slow down" }]));
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn surfaces_response_failed_details_from_response_error() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.events, json!([{ "type": "provider-error", "message": "server_error: Upstream model unavailable" }]));
}

#[test]
#[ignore = "porting: openai responses protocol not implemented"]
fn fails_http_provider_errors_before_stream_parsing() {
    let error = LLMClient::generate(request()).expect_err("should fail");

    assert!(error.to_string().contains("HTTP 400"));
}
