//! Port of packages/llm/test/tool-runtime.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the tool runtime dispatch/loop.
//! Effect-ts `Layer`/`Stream` plumbing is dropped; event and projection
//! behaviour is kept.

use opencode_llm::{LLMEvent, LLMResponse, Tool, ToolRuntime};
use serde_json::json;

fn get_weather() -> serde_json::Value {
    Tool::make(json!({
        "description": "Get current weather for a city.",
        "parameters": { "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] },
        "success": { "type": "object", "properties": { "temperature": { "type": "number" }, "condition": { "type": "string" } }, "required": ["temperature", "condition"] },
    }))
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn uses_the_registered_model_route_when_adding_runtime_tools() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "canned": { "text": "Done." },
    }))
    .expect("runTools");

    assert_eq!(LLMResponse::text(&events), "Done.");
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn dispatches_a_tool_call_appends_results_and_resumes_streaming() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "canned": { "toolCall": { "id": "call_1", "name": "get_weather", "input": { "city": "Paris" } }, "result": { "temperature": 22, "condition": "sunny" }, "text": "It's sunny in Paris." },
    }))
    .expect("runTools");

    let result = events
        .iter()
        .find(|event| LLMEvent::is_tool_result(event))
        .expect("tool-result");
    assert_eq!(result["id"], "call_1");
    assert_eq!(result["name"], "get_weather");
    assert_eq!(
        result["result"],
        json!({ "type": "json", "value": { "temperature": 22, "condition": "sunny" } })
    );
    assert_eq!(events.last().expect("finish")["type"], "finish");
    assert_eq!(LLMResponse::text(&events), "It's sunny in Paris.");
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn projects_encoded_typed_tool_success_into_canonical_model_content() {
    let dispatched = ToolRuntime::dispatch(
        json!({ "projected": Tool::make(json!({ "description": "Project an encoded success." })) }),
        json!({ "id": "call_projected", "name": "projected", "input": { "prefix": "count" } }),
    )
    .expect("dispatch");

    assert_eq!(
        dispatched["result"],
        json!({ "type": "text", "value": "count:2" })
    );
    assert_eq!(
        dispatched["output"],
        json!({ "structured": { "count": "2" }, "content": [{ "type": "text", "text": "count:2" }] })
    );
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn uses_the_narrow_default_projection_for_encoded_typed_success() {
    let text = ToolRuntime::dispatch(
        json!({ "text": Tool::make(json!({ "description": "Return text." })) }),
        json!({ "id": "call_text", "name": "text", "input": {} }),
    )
    .expect("dispatch");
    assert_eq!(
        text["output"],
        json!({ "structured": "hello", "content": [{ "type": "text", "text": "hello" }] })
    );

    let json_tool = ToolRuntime::dispatch(
        json!({ "json": Tool::make(json!({ "description": "Return JSON." })) }),
        json!({ "id": "call_json", "name": "json", "input": {} }),
    )
    .expect("dispatch");
    assert_eq!(
        json_tool["output"],
        json!({ "structured": { "ok": true }, "content": [] })
    );
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn can_retain_model_media_while_redacting_duplicated_structured_payloads() {
    let dispatched = ToolRuntime::dispatch(
        json!({ "image": Tool::make(json!({ "description": "Return an image." })) }),
        json!({ "id": "call_image", "name": "image", "input": {} }),
    )
    .expect("dispatch");

    assert_eq!(
        dispatched["output"],
        json!({
            "structured": { "mime": "image/png" },
            "content": [{ "type": "file", "uri": "data:image/png;base64,AAECAw==", "mime": "image/png" }]
        })
    );
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn settles_projected_url_files_as_canonical_tool_results() {
    let dispatched = ToolRuntime::dispatch(
        json!({ "remote": Tool::make(json!({ "description": "Return a remote file." })) }),
        json!({ "id": "call_remote", "name": "remote", "input": {} }),
    )
    .expect("dispatch");

    assert_eq!(
        dispatched["result"],
        json!({ "type": "content", "value": [{ "type": "file", "uri": "https://example.test/image.png", "mime": "image/png" }] })
    );
    let types: Vec<&serde_json::Value> = dispatched["events"]
        .as_array()
        .expect("events")
        .iter()
        .map(|event| &event["type"])
        .collect();
    assert_eq!(types, vec![&json!("tool-result")]);
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn executes_tool_calls_for_one_step_without_looping_by_default() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "maxSteps": 1,
        "canned": { "toolCall": { "id": "call_1", "name": "get_weather", "input": { "city": "Paris" } } },
    }))
    .expect("runTools");

    assert_eq!(
        events
            .iter()
            .filter(|event| LLMEvent::is_finish(event))
            .count(),
        1
    );
    assert!(events
        .iter()
        .any(|event| LLMEvent::is_tool_result(event) && event["id"] == "call_1"));
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn emits_tool_error_for_unknown_tools_so_the_model_can_self_correct() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "canned": { "toolCall": { "id": "call_1", "name": "missing_tool", "input": {} }, "text": "Sorry." },
    }))
    .expect("runTools");

    let tool_error = events
        .iter()
        .find(|event| LLMEvent::is_tool_error(event))
        .expect("tool-error");
    assert_eq!(tool_error["id"], "call_1");
    assert_eq!(tool_error["name"], "missing_tool");
    assert!(tool_error["message"]
        .as_str()
        .unwrap_or_default()
        .contains("Unknown tool"));
    let result = events
        .iter()
        .find(|event| LLMEvent::is_tool_result(event))
        .expect("tool-result");
    assert_eq!(
        result["result"],
        json!({ "type": "error", "value": "Unknown tool: missing_tool" })
    );
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn emits_tool_error_when_the_llm_input_fails_the_parameters_schema() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "canned": { "toolCall": { "id": "call_1", "name": "get_weather", "input": { "city": 42 } }, "text": "Done." },
    }))
    .expect("runTools");

    let tool_error = events
        .iter()
        .find(|event| LLMEvent::is_tool_error(event))
        .expect("tool-error");
    assert!(tool_error["message"]
        .as_str()
        .unwrap_or_default()
        .contains("Invalid tool input"));
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn respects_max_steps_and_stops_the_loop() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "maxSteps": 2,
        "canned": { "repeatToolCall": { "id": "call_x", "name": "get_weather", "input": { "city": "Paris" } } },
    }))
    .expect("runTools");

    assert_eq!(
        events
            .iter()
            .filter(|event| LLMEvent::is_finish(event))
            .count(),
        1
    );
    let starts: Vec<&serde_json::Value> = events
        .iter()
        .filter(|event| LLMEvent::is_step_start(event))
        .map(|event| &event["index"])
        .collect();
    assert_eq!(starts, vec![&json!(0), &json!(1)]);
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn does_not_dispatch_provider_executed_tool_calls() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "claude-sonnet-4-5", "provider": "anthropic", "route": { "id": "anthropic-messages" } }, "prompt": "Use the tool." },
        "tools": {},
        "providerExecuted": true,
    }))
    .expect("runTools");

    assert!(!events.iter().any(LLMEvent::is_tool_error));
    assert_eq!(
        events.iter().find(|event| LLMEvent::is_tool_call(event)),
        Some(
            &json!({ "type": "tool-call", "id": "srvtoolu_abc", "name": "web_search", "input": { "query": "x" }, "providerExecuted": true })
        )
    );
    assert_eq!(LLMResponse::text(&events), "Done.");
}

#[test]
#[ignore = "porting: tool runtime not implemented"]
fn dispatches_multiple_tool_calls_in_one_step_concurrently() {
    let events = ToolRuntime::run(json!({
        "request": { "id": "req_1", "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" } }, "prompt": "Use the tool." },
        "tools": { "get_weather": get_weather() },
        "canned": { "toolCalls": [
            { "id": "c1", "name": "get_weather", "input": { "city": "Paris" } },
            { "id": "c2", "name": "get_weather", "input": { "city": "Tokyo" } }
        ] },
    }))
    .expect("runTools");

    let mut ids: Vec<String> = events
        .iter()
        .filter(|event| LLMEvent::is_tool_result(event))
        .filter_map(|event| event["id"].as_str().map(str::to_string))
        .collect();
    assert_eq!(ids.len(), 2);
    ids.sort();
    assert_eq!(ids, vec!["c1".to_string(), "c2".to_string()]);
}
