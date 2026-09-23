#![allow(dead_code)]

//! Port of packages/opencode/test/session/llm.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `session.llm.hasToolCalls` and the pure AI-SDK adapter
//! chunk fold (`session.llm.ai-sdk adapter`): session-visible event mapping,
//! non-visible chunk filtering, stable block ids, undefined-usage signalling,
//! and provider metadata preservation on step-finish. The same file's
//! `session.llm.stream` cases need a live provider HTTP runtime and are dropped
//! here (noted in PORT-STATUS.s2.json). Stubs are local per the fast-wave
//! protocol.

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContentPiece {
    Text,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MsgContent {
    Text,
    Parts(Vec<ContentPiece>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelMessage {
    content: MsgContent,
}

fn has_tool_calls(messages: &[ModelMessage]) -> Result<bool, S2Error> {
    Ok(messages.iter().any(|message| match &message.content {
        MsgContent::Text => false,
        MsgContent::Parts(pieces) => pieces
            .iter()
            .any(|piece| matches!(piece, ContentPiece::ToolCall | ContentPiece::ToolResult)),
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LlmUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_tokens: Option<u64>,
    reasoning_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
    cache_write_input_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
enum Chunk {
    TextStart {
        id: Option<String>,
        metadata: Option<Value>,
    },
    TextDelta {
        id: Option<String>,
        text: String,
        metadata: Option<Value>,
    },
    TextEnd {
        id: Option<String>,
        metadata: Option<Value>,
    },
    ReasoningStart {
        id: Option<String>,
        metadata: Option<Value>,
    },
    ReasoningDelta {
        id: Option<String>,
        text: String,
        metadata: Option<Value>,
    },
    ReasoningEnd {
        id: Option<String>,
        metadata: Option<Value>,
    },
    ToolInputStart {
        id: String,
        name: String,
        metadata: Option<Value>,
    },
    ToolInputDelta {
        id: String,
        name: String,
        delta: String,
    },
    ToolInputEnd {
        id: String,
        name: String,
        metadata: Option<Value>,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
        metadata: Option<Value>,
    },
    ToolResult {
        id: String,
        name: String,
        result: Value,
        metadata: Option<Value>,
    },
    StepStart,
    StepFinish {
        reason: String,
        usage: Option<LlmUsage>,
        metadata: Option<Value>,
    },
    Finish {
        reason: String,
        usage: Option<LlmUsage>,
    },
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
enum LlmEvent {
    StepStart {
        index: u64,
    },
    TextStart {
        id: String,
        metadata: Option<Value>,
    },
    TextDelta {
        id: String,
        text: String,
        metadata: Option<Value>,
    },
    TextEnd {
        id: String,
        metadata: Option<Value>,
    },
    ReasoningStart {
        id: String,
        metadata: Option<Value>,
    },
    ReasoningDelta {
        id: String,
        text: String,
        metadata: Option<Value>,
    },
    ReasoningEnd {
        id: String,
        metadata: Option<Value>,
    },
    ToolInputStart {
        id: String,
        name: String,
        metadata: Option<Value>,
    },
    ToolInputDelta {
        id: String,
        name: String,
        text: String,
    },
    ToolInputEnd {
        id: String,
        name: String,
        metadata: Option<Value>,
    },
    ToolCall {
        id: String,
        name: String,
        input: Value,
        metadata: Option<Value>,
    },
    ToolResult {
        id: String,
        name: String,
        result: Value,
        metadata: Option<Value>,
    },
    StepFinish {
        index: u64,
        reason: String,
        usage: Option<LlmUsage>,
        metadata: Option<Value>,
    },
    Finish {
        reason: String,
        usage: Option<LlmUsage>,
    },
}

#[derive(Debug, Default, Clone)]
struct AdapterState {
    text_index: u64,
    reasoning_index: u64,
    step_index: u64,
    current_text_id: Option<String>,
    current_reasoning_id: Option<String>,
}

fn finish_reason(reason: &str) -> String {
    match reason {
        "stop" | "length" | "tool-calls" | "content-filter" | "error" => reason.to_string(),
        _ => "unknown".to_string(),
    }
}

fn usage(value: &Option<LlmUsage>) -> Option<LlmUsage> {
    let usage = value.as_ref()?;
    let any = usage.input_tokens.is_some()
        || usage.output_tokens.is_some()
        || usage.total_tokens.is_some()
        || usage.reasoning_tokens.is_some()
        || usage.cache_read_input_tokens.is_some()
        || usage.cache_write_input_tokens.is_some();
    if any {
        Some(usage.clone())
    } else {
        None
    }
}

fn current_text_id(state: &mut AdapterState, id: Option<String>) -> String {
    if let Some(id) = id {
        state.current_text_id = Some(id);
    } else if state.current_text_id.is_none() {
        let id = format!("text-{}", state.text_index);
        state.text_index += 1;
        state.current_text_id = Some(id);
    }
    state.current_text_id.clone().unwrap_or_default()
}

fn current_reasoning_id(state: &mut AdapterState, id: Option<String>) -> String {
    if let Some(id) = id {
        state.current_reasoning_id = Some(id);
    } else if state.current_reasoning_id.is_none() {
        let id = format!("reasoning-{}", state.reasoning_index);
        state.reasoning_index += 1;
        state.current_reasoning_id = Some(id);
    }
    state.current_reasoning_id.clone().unwrap_or_default()
}

fn to_llm_events(state: &mut AdapterState, chunks: &[Chunk]) -> Result<Vec<LlmEvent>, S2Error> {
    let mut events = Vec::new();
    for chunk in chunks {
        match chunk {
            Chunk::StepStart => events.push(LlmEvent::StepStart {
                index: state.step_index,
            }),
            Chunk::TextStart { id, metadata } => {
                let id = current_text_id(state, id.clone());
                events.push(LlmEvent::TextStart {
                    id,
                    metadata: metadata.clone(),
                });
            }
            Chunk::TextDelta { id, text, metadata } => {
                let id = current_text_id(state, id.clone());
                events.push(LlmEvent::TextDelta {
                    id,
                    text: text.clone(),
                    metadata: metadata.clone(),
                });
            }
            Chunk::TextEnd { id, metadata } => {
                let id = current_text_id(state, id.clone());
                state.current_text_id = None;
                events.push(LlmEvent::TextEnd {
                    id,
                    metadata: metadata.clone(),
                });
            }
            Chunk::ReasoningStart { id, metadata } => {
                let id = current_reasoning_id(state, id.clone());
                events.push(LlmEvent::ReasoningStart {
                    id,
                    metadata: metadata.clone(),
                });
            }
            Chunk::ReasoningDelta { id, text, metadata } => {
                let id = current_reasoning_id(state, id.clone());
                events.push(LlmEvent::ReasoningDelta {
                    id,
                    text: text.clone(),
                    metadata: metadata.clone(),
                });
            }
            Chunk::ReasoningEnd { id, metadata } => {
                let id = current_reasoning_id(state, id.clone());
                state.current_reasoning_id = None;
                events.push(LlmEvent::ReasoningEnd {
                    id,
                    metadata: metadata.clone(),
                });
            }
            Chunk::ToolInputStart { id, name, metadata } => events.push(LlmEvent::ToolInputStart {
                id: id.clone(),
                name: name.clone(),
                metadata: metadata.clone(),
            }),
            Chunk::ToolInputDelta { id, name, delta } => events.push(LlmEvent::ToolInputDelta {
                id: id.clone(),
                name: name.clone(),
                text: delta.clone(),
            }),
            Chunk::ToolInputEnd { id, name, metadata } => events.push(LlmEvent::ToolInputEnd {
                id: id.clone(),
                name: name.clone(),
                metadata: metadata.clone(),
            }),
            Chunk::ToolCall {
                id,
                name,
                input,
                metadata,
            } => events.push(LlmEvent::ToolCall {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
                metadata: metadata.clone(),
            }),
            Chunk::ToolResult {
                id,
                name,
                result,
                metadata,
            } => events.push(LlmEvent::ToolResult {
                id: id.clone(),
                name: name.clone(),
                result: result.clone(),
                metadata: metadata.clone(),
            }),
            Chunk::StepFinish {
                reason,
                usage: chunk_usage,
                metadata,
            } => {
                events.push(LlmEvent::StepFinish {
                    index: state.step_index,
                    reason: finish_reason(reason),
                    usage: usage(chunk_usage),
                    metadata: metadata.clone(),
                });
                state.step_index += 1;
            }
            Chunk::Finish {
                reason,
                usage: chunk_usage,
            } => events.push(LlmEvent::Finish {
                reason: finish_reason(reason),
                usage: usage(chunk_usage),
            }),
            Chunk::Ignored => {}
        }
    }
    Ok(events)
}

fn parts(pieces: &[ContentPiece]) -> ModelMessage {
    ModelMessage {
        content: MsgContent::Parts(pieces.to_vec()),
    }
}

fn text_msg() -> ModelMessage {
    ModelMessage {
        content: MsgContent::Text,
    }
}

#[test]
fn has_tool_calls_returns_false_for_empty_messages() {
    assert!(!has_tool_calls(&[]).expect("hasToolCalls"));
}

#[test]
fn has_tool_calls_returns_false_for_text_only_messages() {
    let messages = vec![parts(&[ContentPiece::Text]), parts(&[ContentPiece::Text])];
    assert!(!has_tool_calls(&messages).expect("hasToolCalls"));
}

#[test]
fn has_tool_calls_returns_true_when_messages_contain_tool_call() {
    let messages = vec![
        parts(&[ContentPiece::Text]),
        parts(&[ContentPiece::ToolCall]),
    ];
    assert!(has_tool_calls(&messages).expect("hasToolCalls"));
}

#[test]
fn has_tool_calls_returns_true_when_messages_contain_tool_result() {
    let messages = vec![parts(&[ContentPiece::ToolResult])];
    assert!(has_tool_calls(&messages).expect("hasToolCalls"));
}

#[test]
fn has_tool_calls_returns_false_for_string_content() {
    let messages = vec![text_msg(), text_msg()];
    assert!(!has_tool_calls(&messages).expect("hasToolCalls"));
}

#[test]
fn has_tool_calls_returns_true_when_tool_call_is_mixed_with_text() {
    let messages = vec![parts(&[ContentPiece::Text, ContentPiece::ToolCall])];
    assert!(has_tool_calls(&messages).expect("hasToolCalls"));
}

#[test]
fn maps_ai_sdk_stream_chunks_without_losing_session_visible_fields() {
    let mut state = AdapterState::default();
    let metadata = Some(json!({ "openai": { "itemID": "item-1" } }));

    let chunks = vec![
        Chunk::StepStart,
        Chunk::TextStart {
            id: Some("text-1".to_string()),
            metadata: metadata.clone(),
        },
        Chunk::TextDelta {
            id: Some("text-1".to_string()),
            text: "Hel".to_string(),
            metadata: Some(json!({ "openai": { "delta": 1 } })),
        },
        Chunk::TextDelta {
            id: Some("text-1".to_string()),
            text: "lo".to_string(),
            metadata: Some(json!({ "openai": { "delta": 2 } })),
        },
        Chunk::TextEnd {
            id: Some("text-1".to_string()),
            metadata: Some(json!({ "openai": { "done": true } })),
        },
        Chunk::ReasoningStart {
            id: Some("reasoning-1".to_string()),
            metadata: metadata.clone(),
        },
        Chunk::ReasoningDelta {
            id: Some("reasoning-1".to_string()),
            text: "Think".to_string(),
            metadata: Some(json!({ "openai": { "delta": 3 } })),
        },
        Chunk::ReasoningEnd {
            id: Some("reasoning-1".to_string()),
            metadata: Some(json!({ "openai": { "done": true } })),
        },
        Chunk::ToolInputStart {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            metadata: metadata.clone(),
        },
        Chunk::ToolInputDelta {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            delta: "{\"query\":".to_string(),
        },
        Chunk::ToolInputDelta {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            delta: "\"weather\"}".to_string(),
        },
        Chunk::ToolInputEnd {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            metadata: Some(json!({ "openai": { "inputDone": true } })),
        },
        Chunk::ToolCall {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            input: json!({ "query": "weather" }),
            metadata: Some(json!({ "openai": { "called": true } })),
        },
        Chunk::ToolResult {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            result: json!({
                "type": "json",
                "value": { "title": "Lookup", "output": "sunny", "metadata": { "ok": true } }
            }),
            metadata: Some(json!({ "openai": { "result": true } })),
        },
        Chunk::StepFinish {
            reason: "other".to_string(),
            usage: Some(LlmUsage {
                input_tokens: Some(10),
                output_tokens: Some(5),
                total_tokens: Some(15),
                reasoning_tokens: Some(1),
                cache_read_input_tokens: Some(3),
                cache_write_input_tokens: Some(2),
            }),
            metadata: Some(json!({ "openai": { "step": true } })),
        },
        Chunk::Finish {
            reason: "other".to_string(),
            usage: Some(LlmUsage {
                input_tokens: Some(11),
                output_tokens: Some(6),
                total_tokens: Some(17),
                reasoning_tokens: Some(2),
                cache_read_input_tokens: Some(4),
                cache_write_input_tokens: None,
            }),
        },
    ];

    let events = to_llm_events(&mut state, &chunks).expect("adapter");
    let expected = vec![
        LlmEvent::StepStart { index: 0 },
        LlmEvent::TextStart {
            id: "text-1".to_string(),
            metadata: metadata.clone(),
        },
        LlmEvent::TextDelta {
            id: "text-1".to_string(),
            text: "Hel".to_string(),
            metadata: Some(json!({ "openai": { "delta": 1 } })),
        },
        LlmEvent::TextDelta {
            id: "text-1".to_string(),
            text: "lo".to_string(),
            metadata: Some(json!({ "openai": { "delta": 2 } })),
        },
        LlmEvent::TextEnd {
            id: "text-1".to_string(),
            metadata: Some(json!({ "openai": { "done": true } })),
        },
        LlmEvent::ReasoningStart {
            id: "reasoning-1".to_string(),
            metadata: metadata.clone(),
        },
        LlmEvent::ReasoningDelta {
            id: "reasoning-1".to_string(),
            text: "Think".to_string(),
            metadata: Some(json!({ "openai": { "delta": 3 } })),
        },
        LlmEvent::ReasoningEnd {
            id: "reasoning-1".to_string(),
            metadata: Some(json!({ "openai": { "done": true } })),
        },
        LlmEvent::ToolInputStart {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            metadata: metadata.clone(),
        },
        LlmEvent::ToolInputDelta {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            text: "{\"query\":".to_string(),
        },
        LlmEvent::ToolInputDelta {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            text: "\"weather\"}".to_string(),
        },
        LlmEvent::ToolInputEnd {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            metadata: Some(json!({ "openai": { "inputDone": true } })),
        },
        LlmEvent::ToolCall {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            input: json!({ "query": "weather" }),
            metadata: Some(json!({ "openai": { "called": true } })),
        },
        LlmEvent::ToolResult {
            id: "call-1".to_string(),
            name: "lookup".to_string(),
            result: json!({
                "type": "json",
                "value": { "title": "Lookup", "output": "sunny", "metadata": { "ok": true } }
            }),
            metadata: Some(json!({ "openai": { "result": true } })),
        },
        LlmEvent::StepFinish {
            index: 0,
            reason: "unknown".to_string(),
            usage: Some(LlmUsage {
                input_tokens: Some(10),
                output_tokens: Some(5),
                total_tokens: Some(15),
                reasoning_tokens: Some(1),
                cache_read_input_tokens: Some(3),
                cache_write_input_tokens: Some(2),
            }),
            metadata: Some(json!({ "openai": { "step": true } })),
        },
        LlmEvent::Finish {
            reason: "unknown".to_string(),
            usage: Some(LlmUsage {
                input_tokens: Some(11),
                output_tokens: Some(6),
                total_tokens: Some(17),
                reasoning_tokens: Some(2),
                cache_read_input_tokens: Some(4),
                cache_write_input_tokens: None,
            }),
        },
    ];

    assert_eq!(events, expected);
}

#[test]
fn creates_stable_block_ids_when_ai_sdk_omits_them() {
    let mut state = AdapterState::default();
    let chunks = vec![
        Chunk::TextDelta {
            id: None,
            text: "implicit text".to_string(),
            metadata: None,
        },
        Chunk::TextEnd {
            id: None,
            metadata: None,
        },
        Chunk::ReasoningDelta {
            id: None,
            text: "implicit reasoning".to_string(),
            metadata: None,
        },
        Chunk::ReasoningEnd {
            id: None,
            metadata: None,
        },
    ];

    let events = to_llm_events(&mut state, &chunks).expect("adapter");
    assert_eq!(
        events,
        vec![
            LlmEvent::TextDelta {
                id: "text-0".to_string(),
                text: "implicit text".to_string(),
                metadata: None,
            },
            LlmEvent::TextEnd {
                id: "text-0".to_string(),
                metadata: None,
            },
            LlmEvent::ReasoningDelta {
                id: "reasoning-0".to_string(),
                text: "implicit reasoning".to_string(),
                metadata: None,
            },
            LlmEvent::ReasoningEnd {
                id: "reasoning-0".to_string(),
                metadata: None,
            },
        ]
    );
}

#[test]
fn explicitly_ignores_non_session_visible_ai_sdk_chunks() {
    let mut state = AdapterState::default();
    let chunks = vec![
        Chunk::Ignored,
        Chunk::Ignored,
        Chunk::Ignored,
        Chunk::Ignored,
        Chunk::Ignored,
        Chunk::Ignored,
    ];
    let events = to_llm_events(&mut state, &chunks).expect("adapter");
    assert!(events.is_empty());
}

#[test]
fn emits_undefined_usage_when_every_usage_field_is_missing() {
    let mut state = AdapterState::default();
    let chunks = vec![Chunk::StepFinish {
        reason: "stop".to_string(),
        usage: Some(LlmUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            reasoning_tokens: None,
            cache_read_input_tokens: None,
            cache_write_input_tokens: None,
        }),
        metadata: None,
    }];

    let events = to_llm_events(&mut state, &chunks).expect("adapter");
    assert_eq!(events.len(), 1);
    match &events[0] {
        LlmEvent::StepFinish { usage, .. } => assert!(usage.is_none()),
        other => panic!("expected step-finish, got {other:?}"),
    }
}

#[test]
fn preserves_provider_metadata_on_step_finish_for_anthropic_cache_writes() {
    let mut state = AdapterState::default();
    let provider_metadata = json!({ "anthropic": { "cacheCreationInputTokens": 300 } });
    let chunks = vec![Chunk::StepFinish {
        reason: "stop".to_string(),
        usage: Some(LlmUsage {
            input_tokens: Some(1000),
            output_tokens: Some(500),
            total_tokens: Some(1500),
            reasoning_tokens: None,
            cache_read_input_tokens: Some(200),
            cache_write_input_tokens: None,
        }),
        metadata: Some(provider_metadata.clone()),
    }];

    let events = to_llm_events(&mut state, &chunks).expect("adapter");
    assert_eq!(events.len(), 1);
    match &events[0] {
        LlmEvent::StepFinish {
            usage, metadata, ..
        } => {
            assert_eq!(metadata.as_ref(), Some(&provider_metadata));
            let usage = usage.as_ref().expect("usage");
            assert!(usage.cache_write_input_tokens.is_none());
            assert_eq!(usage.cache_read_input_tokens, Some(200));
        }
        other => panic!("expected step-finish, got {other:?}"),
    }
}
