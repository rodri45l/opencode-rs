//! Port of packages/llm/test/tool-stream.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `ToolStream`.

use opencode_llm::ToolStream;
use serde_json::json;

#[test]
fn starts_from_openai_style_deltas_and_finalizes_parsed_input() {
    let first = ToolStream::append_or_start(
        ToolStream::empty(),
        0,
        json!({ "id": "call_1", "name": "lookup", "text": "{\"query\"" }),
        "missing tool",
    )
    .expect("appendOrStart");

    assert_eq!(
        first["events"],
        json!([
            { "type": "tool-input-start", "id": "call_1", "name": "lookup" },
            { "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": "{\"query\"" }
        ])
    );

    let second =
        ToolStream::append_or_start(first, 0, json!({ "text": ":\"weather\"}" }), "missing tool")
            .expect("appendOrStart");
    assert_eq!(
        second["events"],
        json!([{ "type": "tool-input-delta", "id": "call_1", "name": "lookup", "text": ":\"weather\"}" }])
    );

    let finished = ToolStream::finish(second, 0).expect("finish");
    assert_eq!(
        finished,
        json!({
            "tools": {},
            "events": [
                { "type": "tool-input-end", "id": "call_1", "name": "lookup" },
                { "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "weather" } }
            ]
        })
    );
}

#[test]
fn fails_append_existing_when_the_provider_skipped_the_tool_start() {
    let error = ToolStream::append_existing(ToolStream::empty(), 0, "{}", "missing tool")
        .expect_err("should fail");

    assert!(error.to_string().contains("missing tool"), "got: {error}");
}

#[test]
fn uses_final_input_override_without_losing_accumulated_deltas() {
    let tools = ToolStream::start(
        ToolStream::empty(),
        json!("item_1"),
        json!({ "id": "call_1", "name": "lookup", "input": "{\"query\":\"partial\"}" }),
    );
    let finished = ToolStream::finish_with_input(tools, json!("item_1"), "{\"query\":\"final\"}")
        .expect("finishWithInput");

    assert_eq!(
        finished,
        json!({
            "tools": {},
            "events": [
                { "type": "tool-input-end", "id": "call_1", "name": "lookup" },
                { "type": "tool-call", "id": "call_1", "name": "lookup", "input": { "query": "final" } }
            ]
        })
    );
}

#[test]
fn preserves_provider_executed_and_clears_all_tools() {
    let first = ToolStream::start(
        ToolStream::empty(),
        json!(0),
        json!({ "id": "call_1", "name": "lookup", "input": "{}" }),
    );
    let tools = ToolStream::start(
        first,
        json!(1),
        json!({ "id": "call_2", "name": "web_search", "input": "{\"query\":\"docs\"}", "providerExecuted": true }),
    );
    let finished = ToolStream::finish_all(tools).expect("finishAll");

    assert_eq!(
        finished,
        json!({
            "tools": {},
            "events": [
                { "type": "tool-input-end", "id": "call_1", "name": "lookup" },
                { "type": "tool-call", "id": "call_1", "name": "lookup", "input": {} },
                { "type": "tool-input-end", "id": "call_2", "name": "web_search" },
                { "type": "tool-call", "id": "call_2", "name": "web_search", "input": { "query": "docs" }, "providerExecuted": true }
            ]
        })
    );
}
