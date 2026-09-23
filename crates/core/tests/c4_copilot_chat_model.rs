//! Port of packages/core/test/github-copilot/copilot-chat-model.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: OpenAI-compatible chat chunks stream `text-start` /
//! `text-delta` / `text-end` around `finish`; reasoning text streams
//! `reasoning-start` / `reasoning-delta` / `reasoning-end` with `reasoning-end`
//! emitted before `text-start` or `tool-input-start`; `reasoning_opaque`
//! attaches to `reasoning-end` when it arrives early and to `finish` when it
//! arrives late; tool calls emit `tool-input-start` plus `tool-call` and carry
//! `reasoning_opaque` when there is no reasoning text; response metadata comes
//! from the first chunk; `stream-start` carries warnings; raw chunks are
//! included on request; and request tools are serialized in OpenAI function
//! format. Re-derived: the live runtime is replaced by a pure chunk parser in
//! `opencode_core::copilot_chat_model`.

use opencode_core::copilot_chat_model::{CopilotChatModel, StreamPart};
use serde_json::{json, Value};

fn text_chunk(id: &str, model: &str, content: &str, finish: Option<&str>) -> Value {
    json!({
        "id": id,
        "model": model,
        "choices": [{ "index": 0, "delta": { "content": content }, "finish_reason": finish }]
    })
}

fn reasoning_chunk(text: &str) -> Value {
    json!({
        "choices": [{ "index": 0, "delta": { "content": null, "reasoning_text": text }, "finish_reason": null }]
    })
}

fn tool_chunk(
    id: &str,
    name: &str,
    args: &str,
    index: u32,
    opaque: Option<&str>,
    finish: Option<&str>,
) -> Value {
    let mut delta = json!({
        "content": null,
        "tool_calls": [{
            "id": id,
            "index": index,
            "type": "function",
            "function": { "name": name, "arguments": args }
        }]
    });
    if let Some(opaque) = opaque {
        delta["reasoning_opaque"] = json!(opaque);
    }
    json!({
        "id": "chatcmpl",
        "model": "gemini-3-pro-preview",
        "usage": { "prompt_tokens": 19581, "completion_tokens": 53 },
        "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }]
    })
}

fn filter_text(parts: &[StreamPart]) -> Vec<StreamPart> {
    parts
        .iter()
        .filter(|part| {
            matches!(
                part,
                StreamPart::TextStart { .. }
                    | StreamPart::TextDelta { .. }
                    | StreamPart::TextEnd { .. }
                    | StreamPart::Finish { .. }
            )
        })
        .cloned()
        .collect()
}

fn find_finish(parts: &[StreamPart]) -> StreamPart {
    parts
        .iter()
        .find(|part| matches!(part, StreamPart::Finish { .. }))
        .cloned()
        .expect("finish")
}

#[test]
fn streams_text_deltas() {
    let parts = CopilotChatModel::parse(
        &[
            text_chunk("chatcmpl-123", "gemini-2.0-flash-001", "Hello", None),
            text_chunk("chatcmpl-123", "gemini-2.0-flash-001", " world", None),
            text_chunk("chatcmpl-123", "gemini-2.0-flash-001", "!", Some("stop")),
        ],
        false,
    );

    assert_eq!(
        filter_text(&parts),
        vec![
            StreamPart::TextStart { id: "txt-0".into() },
            StreamPart::TextDelta {
                id: "txt-0".into(),
                delta: "Hello".into()
            },
            StreamPart::TextDelta {
                id: "txt-0".into(),
                delta: " world".into()
            },
            StreamPart::TextDelta {
                id: "txt-0".into(),
                delta: "!".into()
            },
            StreamPart::TextEnd { id: "txt-0".into() },
            StreamPart::Finish {
                reason: "stop".into(),
                input_tokens: 0,
                output_tokens: 0,
                reasoning_opaque: None
            },
        ]
    );
}

#[test]
fn streams_reasoning_with_tool_calls_and_captures_reasoning_opaque() {
    let parts = CopilotChatModel::parse(
        &[
            reasoning_chunk("thinking one"),
            tool_chunk(
                "call_abc123",
                "read_file",
                "{\"filePath\":\"/README.md\"}",
                0,
                Some("opaque-early"),
                Some("tool_calls"),
            ),
        ],
        false,
    );

    let reasoning_end = parts
        .iter()
        .find(|part| matches!(part, StreamPart::ReasoningEnd { .. }))
        .cloned()
        .expect("reasoning end");
    assert_eq!(
        reasoning_end,
        StreamPart::ReasoningEnd {
            id: "reasoning-0".into(),
            reasoning_opaque: Some("opaque-early".into())
        }
    );

    let tool_start = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ToolInputStart { .. }))
        .expect("tool start");
    let reasoning_end_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ReasoningEnd { .. }))
        .expect("reasoning end");
    assert!(reasoning_end_index < tool_start);

    assert_eq!(
        find_finish(&parts),
        StreamPart::Finish {
            reason: "tool-calls".into(),
            input_tokens: 19581,
            output_tokens: 53,
            reasoning_opaque: None
        }
    );
}

#[test]
fn attaches_late_reasoning_opaque_to_the_finish_event() {
    let parts = CopilotChatModel::parse(
        &[
            reasoning_chunk("thinking"),
            text_chunk("chatcmpl", "m", "answer", None),
            tool_chunk(
                "call_late",
                "read_file",
                "{}",
                0,
                Some("opaque-late"),
                Some("stop"),
            ),
        ],
        false,
    );

    let reasoning_end_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ReasoningEnd { .. }))
        .expect("reasoning end");
    let text_start_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::TextStart { .. }))
        .expect("text start");
    assert!(reasoning_end_index < text_start_index);

    match find_finish(&parts) {
        StreamPart::Finish {
            reasoning_opaque, ..
        } => {
            assert_eq!(reasoning_opaque.as_deref(), Some("opaque-late"))
        }
        other => panic!("unexpected finish: {other:?}"),
    }
}

#[test]
fn handles_reasoning_opaque_and_content_in_the_same_chunk() {
    let mut chunk = text_chunk("chatcmpl", "m", "Of course.", None);
    chunk["choices"][0]["delta"]["reasoning_opaque"] = json!("opaque-same");
    let parts = CopilotChatModel::parse(&[reasoning_chunk("thinking"), chunk], false);

    let reasoning_end_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ReasoningEnd { .. }))
        .expect("reasoning end");
    let text_start_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::TextStart { .. }))
        .expect("text start");
    assert!(reasoning_end_index < text_start_index);
    assert_eq!(
        parts[reasoning_end_index],
        StreamPart::ReasoningEnd {
            id: "reasoning-0".into(),
            reasoning_opaque: Some("opaque-same".into())
        }
    );
}

#[test]
fn emits_reasoning_end_before_tool_input_start_when_reasoning_goes_directly_to_tools() {
    let parts = CopilotChatModel::parse(
        &[
            reasoning_chunk("thinking"),
            tool_chunk(
                "call_x",
                "project_eval",
                "{}",
                0,
                Some("opaque"),
                Some("tool_calls"),
            ),
        ],
        false,
    );

    let reasoning_end_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ReasoningEnd { .. }))
        .expect("reasoning end");
    let tool_start_index = parts
        .iter()
        .position(|part| matches!(part, StreamPart::ToolInputStart { .. }))
        .expect("tool start");
    assert!(reasoning_end_index < tool_start_index);
    assert!(!parts
        .iter()
        .any(|part| matches!(part, StreamPart::TextStart { .. })));
}

#[test]
fn attaches_reasoning_opaque_to_tool_calls_without_reasoning_text() {
    let parts = CopilotChatModel::parse(
        &[tool_chunk(
            "call_reasoning_only",
            "read_file",
            "{}",
            0,
            Some("opaque-xyz"),
            Some("tool_calls"),
        )],
        false,
    );

    assert!(!parts.iter().any(|part| matches!(
        part,
        StreamPart::ReasoningStart { .. }
            | StreamPart::ReasoningDelta { .. }
            | StreamPart::ReasoningEnd { .. }
    )));
    let tool_call = parts
        .iter()
        .find(|part| matches!(part, StreamPart::ToolCall { .. }))
        .cloned()
        .expect("tool call");
    assert_eq!(
        tool_call,
        StreamPart::ToolCall {
            tool_call_id: "call_reasoning_only".into(),
            tool_name: "read_file".into(),
            input: json!({}),
            reasoning_opaque: Some("opaque-xyz".into())
        }
    );
}

#[test]
fn includes_response_metadata_from_the_first_chunk() {
    let parts = CopilotChatModel::parse(
        &[text_chunk(
            "chatcmpl-123",
            "gemini-2.0-flash-001",
            "Hello",
            None,
        )],
        false,
    );
    assert_eq!(
        parts[0],
        StreamPart::ResponseMetadata {
            id: "chatcmpl-123".into(),
            model_id: "gemini-2.0-flash-001".into()
        }
    );
}

#[test]
fn emits_stream_start_with_warnings_and_raw_chunks_when_requested() {
    let parts = CopilotChatModel::parse(&[text_chunk("id", "m", "Hello", Some("stop"))], true);
    assert!(parts
        .iter()
        .any(|part| matches!(part, StreamPart::StreamStart { warnings } if warnings.is_empty())));
    assert!(parts.iter().any(|part| matches!(part, StreamPart::Raw)));
}

#[test]
fn sends_tools_in_openai_function_format() {
    assert_eq!(
        CopilotChatModel::tool_request(&[json!({
            "type": "function",
            "name": "get_weather",
            "description": "Get the weather for a location",
            "inputSchema": {
                "type": "object",
                "properties": { "location": { "type": "string" } },
                "required": ["location"]
            }
        })]),
        json!({
            "tools": [{
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get the weather for a location",
                    "parameters": {
                        "type": "object",
                        "properties": { "location": { "type": "string" } },
                        "required": ["location"]
                    }
                }
            }]
        })
    );
}
