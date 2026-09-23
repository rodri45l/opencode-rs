//! Copilot chat-model stream parser (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/github-copilot/chat/copilot-chat-model.ts`: OpenAI-
//! compatible chat chunks stream `text-start`/`text-delta`/`text-end` around
//! `finish`; reasoning text streams `reasoning-start`/`reasoning-delta`/
//! `reasoning-end` with `reasoning-end` emitted before `text-start` or
//! `tool-input-start`; `reasoning_opaque` attaches to `reasoning-end` when it
//! arrives early, to the tool call when there is no reasoning text, and to
//! `finish` when it arrives late; tool calls emit `tool-input-start` plus
//! `tool-call`; response metadata comes from the first chunk; `stream-start`
//! carries warnings; raw chunks are included on request; and request tools are
//! serialized in OpenAI function format. The live `fetch`/`ReadableStream` and
//! `@ai-sdk/provider` types are replaced by a pure chunk parser.

use serde_json::{json, Value};

/// A parsed stream part.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamPart {
    /// Stream start with warnings.
    StreamStart {
        /// Warnings.
        warnings: Vec<String>,
    },
    /// Response metadata.
    ResponseMetadata {
        /// Response id.
        id: String,
        /// Model id.
        model_id: String,
    },
    /// Reasoning start.
    ReasoningStart {
        /// Reasoning id.
        id: String,
    },
    /// Reasoning delta.
    ReasoningDelta {
        /// Reasoning id.
        id: String,
        /// Text delta.
        delta: String,
    },
    /// Reasoning end.
    ReasoningEnd {
        /// Reasoning id.
        id: String,
        /// Late reasoning signature.
        reasoning_opaque: Option<String>,
    },
    /// Text start.
    TextStart {
        /// Text id.
        id: String,
    },
    /// Text delta.
    TextDelta {
        /// Text id.
        id: String,
        /// Text delta.
        delta: String,
    },
    /// Text end.
    TextEnd {
        /// Text id.
        id: String,
    },
    /// Tool input start.
    ToolInputStart {
        /// Tool call id.
        id: String,
        /// Tool name.
        tool_name: String,
    },
    /// Tool call.
    ToolCall {
        /// Tool call id.
        tool_call_id: String,
        /// Tool name.
        tool_name: String,
        /// Parsed input.
        input: Value,
        /// Late reasoning signature.
        reasoning_opaque: Option<String>,
    },
    /// Finish.
    Finish {
        /// Finish reason.
        reason: String,
        /// Input tokens.
        input_tokens: u64,
        /// Output tokens.
        output_tokens: u64,
        /// Late reasoning signature.
        reasoning_opaque: Option<String>,
    },
    /// A raw chunk, included on request.
    Raw,
}

/// The Copilot chat-model stream parser.
pub struct CopilotChatModel;

impl CopilotChatModel {
    /// Parse OpenAI-compatible chat chunks into stream parts.
    pub fn parse(chunks: &[Value], include_raw: bool) -> Vec<StreamPart> {
        let mut parts = Vec::new();
        if include_raw {
            parts.push(StreamPart::StreamStart {
                warnings: Vec::new(),
            });
        }
        let mut meta_emitted = false;
        let mut text_active = false;
        let mut reasoning_active = false;
        let mut reasoning_seen = false;
        let mut pending_opaque: Option<String> = None;

        for chunk in chunks {
            if !meta_emitted {
                parts.push(StreamPart::ResponseMetadata {
                    id: chunk
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    model_id: chunk
                        .get("model")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                });
                meta_emitted = true;
            }

            let choice = chunk.get("choices").and_then(|choices| choices.get(0));
            let delta = choice
                .and_then(|choice| choice.get("delta"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let finish = choice
                .and_then(|choice| choice.get("finish_reason"))
                .and_then(Value::as_str)
                .map(str::to_string);

            let mut opaque = delta
                .get("reasoning_opaque")
                .and_then(Value::as_str)
                .map(str::to_string);
            let reasoning_text = delta.get("reasoning_text").and_then(Value::as_str);
            let content = delta.get("content").and_then(Value::as_str);
            let tool_calls = delta.get("tool_calls").and_then(Value::as_array);

            if let Some(reasoning_text) = reasoning_text {
                if !reasoning_active {
                    parts.push(StreamPart::ReasoningStart {
                        id: "reasoning-0".into(),
                    });
                    reasoning_active = true;
                    reasoning_seen = true;
                }
                parts.push(StreamPart::ReasoningDelta {
                    id: "reasoning-0".into(),
                    delta: reasoning_text.to_string(),
                });
            }

            let has_other = content.is_some() || tool_calls.is_some();
            if has_other && reasoning_active {
                parts.push(StreamPart::ReasoningEnd {
                    id: "reasoning-0".into(),
                    reasoning_opaque: opaque.take(),
                });
                reasoning_active = false;
            }

            if let Some(content) = content {
                if !text_active {
                    parts.push(StreamPart::TextStart { id: "txt-0".into() });
                    text_active = true;
                }
                parts.push(StreamPart::TextDelta {
                    id: "txt-0".into(),
                    delta: content.to_string(),
                });
            }

            if let Some(tool_calls) = tool_calls {
                for call in tool_calls {
                    let call_id = call
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let tool_name = call
                        .get("function")
                        .and_then(|function| function.get("name"))
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    parts.push(StreamPart::ToolInputStart {
                        id: call_id.clone(),
                        tool_name: tool_name.clone(),
                    });
                    let input = call
                        .get("function")
                        .and_then(|function| function.get("arguments"))
                        .and_then(Value::as_str)
                        .and_then(|arguments| serde_json::from_str(arguments).ok())
                        .unwrap_or_else(|| json!({}));
                    let call_opaque = if reasoning_seen { None } else { opaque.clone() };
                    parts.push(StreamPart::ToolCall {
                        tool_call_id: call_id,
                        tool_name,
                        input,
                        reasoning_opaque: call_opaque,
                    });
                }
            }

            if let Some(opaque) = opaque {
                if reasoning_seen && !reasoning_active {
                    pending_opaque = Some(opaque);
                }
            }

            if let Some(finish) = finish {
                if text_active {
                    parts.push(StreamPart::TextEnd { id: "txt-0".into() });
                    text_active = false;
                }
                if reasoning_active {
                    parts.push(StreamPart::ReasoningEnd {
                        id: "reasoning-0".into(),
                        reasoning_opaque: pending_opaque.take(),
                    });
                    reasoning_active = false;
                }
                let usage = chunk.get("usage");
                parts.push(StreamPart::Finish {
                    reason: finish.replace('_', "-"),
                    input_tokens: usage
                        .and_then(|usage| usage.get("prompt_tokens"))
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    output_tokens: usage
                        .and_then(|usage| usage.get("completion_tokens"))
                        .and_then(Value::as_u64)
                        .unwrap_or(0),
                    reasoning_opaque: pending_opaque.take(),
                });
            }

            if include_raw {
                parts.push(StreamPart::Raw);
            }
        }

        parts
    }

    /// Serialize tools in OpenAI function format.
    pub fn tool_request(tools: &[Value]) -> Value {
        let serialized: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let mut function = serde_json::Map::new();
                function.insert(
                    "name".into(),
                    tool.get("name").cloned().unwrap_or(Value::Null),
                );
                if let Some(description) = tool.get("description") {
                    function.insert("description".into(), description.clone());
                }
                function.insert(
                    "parameters".into(),
                    tool.get("inputSchema")
                        .cloned()
                        .unwrap_or_else(|| json!({})),
                );
                json!({ "type": "function", "function": Value::Object(function) })
            })
            .collect();
        json!({ "tools": serialized })
    }
}
