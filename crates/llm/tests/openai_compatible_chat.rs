//! Port of packages/llm/test/provider/openai-compatible-chat.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the OpenAI-compatible Chat protocol and provider families.

use opencode_llm::{providers, Auth, LLMClient, Message, ToolCallPart, LLM};
use serde_json::json;

fn model() -> serde_json::Value {
    json!({
        "id": "deepseek-chat",
        "provider": "deepseek",
        "route": { "id": "openai-compatible-chat" },
        "endpoint": { "baseURL": "https://api.deepseek.test/v1/", "query": { "api-version": "2026-01-01" } },
        "auth": Auth::bearer("test-key"),
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
#[ignore = "porting: openai compatible chat protocol not implemented"]
fn prepares_generic_chat_target() {
    let prepared = LLMClient::prepare(LLM::update_request(request(), json!({
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object" } }],
        "toolChoice": { "type": "required" }
    })).expect("update"))
    .expect("prepare");

    assert_eq!(prepared.route, "openai-compatible-chat");
    assert_eq!(prepared.model["id"], "deepseek-chat");
    assert_eq!(prepared.body["model"], "deepseek-chat");
    assert_eq!(prepared.body["tool_choice"], "required");
    assert_eq!(prepared.body["stream"], true);
    assert_eq!(
        prepared.body["stream_options"],
        json!({ "include_usage": true })
    );
}

#[test]
#[ignore = "porting: openai compatible chat protocol not implemented"]
fn provides_model_helpers_for_compatible_provider_families() {
    let families = [
        (
            "baseten",
            providers::openai_compatible::baseten(),
            "https://inference.baseten.co/v1",
        ),
        (
            "cerebras",
            providers::openai_compatible::cerebras(),
            "https://api.cerebras.ai/v1",
        ),
        (
            "deepinfra",
            providers::openai_compatible::deepinfra(),
            "https://api.deepinfra.com/v1/openai",
        ),
        (
            "deepseek",
            providers::openai_compatible::deepseek(),
            "https://api.deepseek.com/v1",
        ),
        (
            "fireworks",
            providers::openai_compatible::fireworks(),
            "https://api.fireworks.ai/inference/v1",
        ),
        (
            "togetherai",
            providers::openai_compatible::togetherai(),
            "https://api.together.xyz/v1",
        ),
    ];

    for (provider, family, base_url) in families {
        let model = family
            .configure(json!({ "apiKey": "test-key" }))
            .model(&format!("{provider}-model"));
        assert_eq!(model["id"], format!("{provider}-model"));
        assert_eq!(model["provider"], provider);
        assert_eq!(model["route"]["id"], "openai-compatible-chat");
        assert_eq!(model["endpoint"]["baseURL"], base_url);
    }
}

#[test]
#[ignore = "porting: openai compatible chat protocol not implemented"]
fn matches_ai_sdk_compatible_tool_request_body_fixture() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "id": "req_tool_parity",
        "model": model(),
        "tools": [{ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object", "properties": { "query": { "type": "string" } }, "required": ["query"] } }],
        "toolChoice": "lookup",
        "messages": [
            Message::user("What is the weather?"),
            Message::assistant([ToolCallPart::make(json!({ "id": "call_1", "name": "lookup", "input": { "query": "weather" } }))]),
            Message::tool(json!({ "id": "call_1", "name": "lookup", "result": { "forecast": "sunny" } })),
        ],
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["messages"],
        json!([
            { "role": "user", "content": "What is the weather?" },
            { "role": "assistant", "content": null, "tool_calls": [{ "id": "call_1", "type": "function", "function": { "name": "lookup", "arguments": "{\"query\":\"weather\"}" } }] },
            { "role": "tool", "tool_call_id": "call_1", "content": "{\"forecast\":\"sunny\"}" }
        ])
    );
    assert_eq!(
        prepared.body["tool_choice"],
        json!({ "type": "function", "function": { "name": "lookup" } })
    );
}

#[test]
#[ignore = "porting: openai compatible chat protocol not implemented"]
fn posts_to_the_configured_compatible_endpoint_and_parses_text_usage() {
    let response = LLMClient::generate(request()).expect("generate");

    assert_eq!(response.text, "Hello!");
    assert_eq!(response.usage["inputTokens"], 5);
    assert_eq!(response.usage["outputTokens"], 2);
    assert_eq!(response.usage["totalTokens"], 7);
    assert_eq!(response.events.last().expect("finish")["reason"], "stop");
}
