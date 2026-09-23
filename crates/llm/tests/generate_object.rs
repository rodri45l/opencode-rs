//! Port of packages/llm/test/generate-object.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `Tool.make` and `LLM.generateObject`.

use opencode_llm::{Tool, LLM};
use serde_json::json;

#[test]
#[ignore = "porting: generate object not implemented"]
fn forwards_json_schema_and_description_through_to_definitions() {
    let json_schema = json!({
        "type": "object",
        "properties": { "city": { "type": "string" } },
        "required": ["city"]
    });
    let lookup = Tool::make(json!({
        "description": "Look up something",
        "jsonSchema": json_schema,
    }));

    let definitions = Tool::to_definitions(json!({ "lookup": lookup })).expect("toDefinitions");
    let definition = &definitions[0];

    assert_eq!(definition["name"], "lookup");
    assert_eq!(definition["description"], "Look up something");
    assert_eq!(definition["inputSchema"], json_schema);
}

#[test]
#[ignore = "porting: generate object not implemented"]
fn execute_receives_the_raw_input_untouched() {
    let tool = Tool::make(json!({ "description": "echo", "jsonSchema": { "type": "object" } }));
    let result = Tool::execute(tool, json!({ "hello": "world" })).expect("execute");

    assert_eq!(result, json!({ "hello": "world" }));
}

#[test]
#[ignore = "porting: generate object not implemented"]
fn forces_a_synthetic_tool_call_and_decodes_the_input() {
    let response = LLM::generate_object(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } },
        "prompt": "Return a structured weather report.",
        "schema": { "type": "object", "properties": { "city": { "type": "string" }, "temp": { "type": "number" } }, "required": ["city", "temp"] },
    }))
    .expect("generateObject");

    assert_eq!(response["object"], json!({ "city": "Paris", "temp": 22 }));
    assert_eq!(
        response["response"]["toolCalls"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        response["bodies"][0]["tool_choice"],
        json!({ "type": "function", "function": { "name": "generate_object" } })
    );
    assert_eq!(
        response["bodies"][0]["tools"].as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
#[ignore = "porting: generate object not implemented"]
fn accepts_a_raw_json_schema_and_returns_the_input_untouched() {
    let response = LLM::generate_object(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } },
        "prompt": "Extract the user.",
        "jsonSchema": { "type": "object", "properties": { "name": { "type": "string" }, "age": { "type": "number" } }, "required": ["name", "age"] },
    }))
    .expect("generateObject");

    assert_eq!(response["object"], json!({ "name": "Ada", "age": 30 }));
    assert_eq!(
        response["bodies"][0]["tools"][0]["function"]["parameters"],
        json!({ "type": "object", "properties": { "name": { "type": "string" }, "age": { "type": "number" } }, "required": ["name", "age"] })
    );
}

#[test]
#[ignore = "porting: generate object not implemented"]
fn fails_when_the_model_does_not_call_the_synthetic_tool() {
    let error = LLM::generate_object(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } },
        "prompt": "Return a structured value.",
        "schema": { "type": "object", "properties": { "value": { "type": "number" } }, "required": ["value"] },
        "canned": { "text": "no thanks", "finishReason": "stop" },
    }))
    .expect_err("should fail");

    assert!(!error.to_string().is_empty());
}

#[test]
#[ignore = "porting: generate object not implemented"]
fn fails_with_a_decode_error_when_the_tool_input_does_not_match_the_schema() {
    let error = LLM::generate_object(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } },
        "prompt": "Return a structured value.",
        "schema": { "type": "object", "properties": { "value": { "type": "number" } }, "required": ["value"] },
        "canned": { "toolInput": { "value": "not-a-number" } },
    }))
    .expect_err("should fail");

    assert!(!error.to_string().is_empty());
}
