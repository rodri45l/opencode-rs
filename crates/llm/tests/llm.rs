//! Port of packages/llm/test/llm.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the ergonomic `LLM.request` constructors and tool choices.

use opencode_llm::{LLMResponse, Message, Model, ToolChoice, ToolDefinition, LLM};
use serde_json::json;

fn chat_route() -> serde_json::Value {
    json!({ "id": "openai-chat" })
}

#[test]
fn builds_canonical_request_fields_from_ergonomic_input() {
    let request = LLM::request(json!({
        "id": "req_1",
        "model": Model::make(json!({ "id": "fake-model", "provider": "fake", "route": chat_route() })),
        "system": "You are concise.",
        "prompt": "Say hello.",
    }));

    assert_eq!(request["model"]["id"], "fake-model");
    assert_eq!(request["messages"][0]["role"], "user");
    assert_eq!(
        request["messages"][0]["content"],
        json!([{ "type": "text", "text": "Say hello." }])
    );
}

#[test]
fn keeps_request_options_separate_from_route_defaults() {
    let request = LLM::request(json!({
        "model": Model::make(json!({
            "id": "fake-model",
            "provider": "fake",
            "route": chat_route(),
            "defaults": { "generation": { "maxTokens": 100, "temperature": 1 } }
        })),
        "prompt": "Say hello.",
        "generation": { "temperature": 0 },
        "providerOptions": { "openai": { "store": true, "metadata": { "request": true } } },
        "http": { "headers": { "x-shared": "request" }, "query": { "request": "1" } },
    }));

    assert_eq!(request["generation"], json!({ "temperature": 0 }));
    assert_eq!(
        request["providerOptions"],
        json!({ "openai": { "store": true, "metadata": { "request": true } } })
    );
}

#[test]
fn builds_tool_choices_from_names_and_tools() {
    let tool = ToolDefinition::make(
        json!({ "name": "lookup", "description": "Lookup data", "inputSchema": { "type": "object" } }),
    );

    assert_eq!(
        ToolChoice::make(json!("lookup")),
        json!({ "type": "tool", "name": "lookup" })
    );
    assert_eq!(
        ToolChoice::named("required"),
        json!({ "type": "tool", "name": "required" })
    );
    assert_eq!(
        ToolChoice::make(tool),
        json!({ "type": "tool", "name": "lookup" })
    );
}

#[test]
fn builds_tool_choice_modes_from_reserved_strings() {
    assert_eq!(ToolChoice::make(json!("auto")), json!({ "type": "auto" }));
    assert_eq!(ToolChoice::make(json!("none")), json!({ "type": "none" }));
    assert_eq!(
        ToolChoice::make(json!("required")),
        json!({ "type": "required" })
    );
}

#[test]
fn builds_chronological_system_updates_separately_from_the_initial_system_prompt() {
    let update = Message::system(
        json!([{ "type": "text", "text": "Use parameterized SQL.", "cache": { "type": "ephemeral" } }]),
    );
    let request = LLM::request(json!({
        "model": Model::make(json!({ "id": "fake-model", "provider": "fake", "route": chat_route() })),
        "system": "Initial operator prompt.",
        "messages": [Message::user("Review this."), update],
    }));

    assert_eq!(
        request["system"],
        json!([{ "type": "text", "text": "Initial operator prompt." }])
    );
    assert_eq!(request["messages"][0]["role"], "user");
    assert_eq!(request["messages"][1]["role"], "system");
}

#[test]
fn extracts_output_text_from_response_events() {
    let events = vec![
        json!({ "type": "text-delta", "id": "text-0", "text": "hi" }),
        json!({ "type": "finish", "reason": "stop" }),
    ];

    assert_eq!(LLMResponse::text(&events), "hi");
}
