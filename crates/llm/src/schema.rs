//! Canonical request/event data used by the ported tests.
//!
//! The Rust port models wire payloads as [`serde_json::Value`]; the helpers here
//! are the ergonomic constructors the reference exposes as schema classes.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// Token accounting for one completion turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    value: Value,
}

impl Usage {
    /// Wrap a raw usage payload.
    pub fn new(input: Value) -> Self {
        Self { value: input }
    }

    fn int(&self, key: &str) -> Option<i64> {
        self.value.get(key).and_then(Value::as_i64)
    }

    /// Output tokens that were not reasoning tokens, clamped at zero.
    pub fn visible_output_tokens(&self) -> i64 {
        let output = self.int("outputTokens").unwrap_or(0);
        let reasoning = self.int("reasoningTokens").unwrap_or(0);
        (output - reasoning).max(0)
    }

    /// Total tokens when the provider reported them.
    pub fn total_tokens(&self) -> Option<i64> {
        self.int("totalTokens")
    }

    /// Underlying payload.
    pub fn as_value(&self) -> &Value {
        &self.value
    }
}

/// Request construction helpers.
pub struct LLMRequest;

impl LLMRequest {
    /// Apply a patch, returning the updated request.
    pub fn update(_base: Value, _patch: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("llm request update"))
    }

    /// Extract the input fields from a canonical request.
    pub fn input(request: Value) -> Value {
        request
    }
}

/// Model construction helpers.
pub struct Model;

impl Model {
    /// Build a model descriptor.
    pub fn make(input: Value) -> Value {
        input
    }

    /// Apply a patch, returning the updated model.
    pub fn update(_base: Value, _patch: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("model update"))
    }

    /// Extract the input fields from a canonical model.
    pub fn input(model: Value) -> Value {
        model
    }
}

/// Message construction helpers.
pub struct Message;

impl Message {
    /// A user message.
    pub fn user(content: impl Into<Value>) -> Value {
        serde_json::json!({ "role": "user", "content": content.into() })
    }

    /// An assistant message.
    pub fn assistant(content: impl Into<Value>) -> Value {
        serde_json::json!({ "role": "assistant", "content": content.into() })
    }

    /// A system message.
    pub fn system(content: impl Into<Value>) -> Value {
        serde_json::json!({ "role": "system", "content": content.into() })
    }

    /// A tool-result message.
    pub fn tool(content: impl Into<Value>) -> Value {
        serde_json::json!({ "role": "tool", "content": content.into() })
    }

    /// Build a message from a raw object.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Event construction helpers.
pub struct LLMEvent;

impl LLMEvent {
    /// A terminal `finish` event.
    pub fn finish(reason: &str, usage: Option<Value>) -> Value {
        serde_json::json!({ "type": "finish", "reason": reason, "usage": usage })
    }

    /// A per-step `step-finish` event.
    pub fn step_finish(index: u64, reason: &str, usage: Option<Value>) -> Value {
        serde_json::json!({ "type": "step-finish", "index": index, "reason": reason, "usage": usage })
    }

    /// A completed tool call.
    pub fn tool_call(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-call".into()));
        }
        value
    }

    /// A tool result.
    pub fn tool_result(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-result".into()));
        }
        value
    }

    /// A streamed text delta.
    pub fn text_delta(id: &str, text: &str) -> Value {
        serde_json::json!({ "type": "text-delta", "id": id, "text": text })
    }

    fn is(event: &Value, kind: &str) -> bool {
        event.get("type").and_then(Value::as_str) == Some(kind)
    }

    /// Whether the event is a terminal `finish`.
    pub fn is_finish(event: &Value) -> bool {
        Self::is(event, "finish")
    }

    /// Whether the event is a `tool-call`.
    pub fn is_tool_call(event: &Value) -> bool {
        Self::is(event, "tool-call")
    }

    /// Whether the event is a `tool-result`.
    pub fn is_tool_result(event: &Value) -> bool {
        Self::is(event, "tool-result")
    }

    /// Whether the event is a `tool-error`.
    pub fn is_tool_error(event: &Value) -> bool {
        Self::is(event, "tool-error")
    }

    /// Whether the event is a `step-start`.
    pub fn is_step_start(event: &Value) -> bool {
        Self::is(event, "step-start")
    }

    /// Whether the event is a `step-finish`.
    pub fn is_step_finish(event: &Value) -> bool {
        Self::is(event, "step-finish")
    }
}

/// Content-part guards.
pub struct ContentPart;

impl ContentPart {
    /// Whether the part is a text part.
    pub fn guards_text(part: &Value) -> bool {
        part.get("type").and_then(Value::as_str) == Some("text")
    }

    /// Whether the part is a media part.
    pub fn guards_media(part: &Value) -> bool {
        part.get("type").and_then(Value::as_str) == Some("media")
    }
}

/// Tool choice helpers.
pub struct ToolChoice;

impl ToolChoice {
    /// Build a tool choice from a name, reserved mode, or tool definition.
    pub fn make(input: Value) -> Value {
        if let Value::String(name) = &input {
            return match name.as_str() {
                "auto" => serde_json::json!({ "type": "auto" }),
                "none" => serde_json::json!({ "type": "none" }),
                "required" => serde_json::json!({ "type": "required" }),
                _ => serde_json::json!({ "type": "tool", "name": name }),
            };
        }
        if let Value::Object(map) = &input {
            if let Some(name) = map.get("name") {
                return serde_json::json!({ "type": "tool", "name": name });
            }
        }
        input
    }

    /// Build a named tool choice.
    pub fn named(name: &str) -> Value {
        serde_json::json!({ "type": "tool", "name": name })
    }
}

/// Tool definition helper.
pub struct ToolDefinition;

impl ToolDefinition {
    /// Build a tool definition.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Tool call part helper.
pub struct ToolCallPart;

impl ToolCallPart {
    /// Build a tool call part.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Tool result part helper.
pub struct ToolResultPart;

impl ToolResultPart {
    /// Build a tool result part.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// A manual cache hint attached to a request part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHint {
    value: Value,
}

impl CacheHint {
    /// Build a cache hint.
    pub fn new(input: Value) -> Self {
        Self { value: input }
    }

    /// Underlying hint payload.
    pub fn as_value(&self) -> &Value {
        &self.value
    }
}

impl serde::Serialize for CacheHint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

/// Response reducer over a normalised event stream.
pub struct LLMResponse;

impl LLMResponse {
    /// Empty reducer state.
    pub fn empty() -> Value {
        serde_json::json!({ "events": [], "message": { "content": [] } })
    }

    /// Fold one event into the state.
    pub fn reduce(state: Value, _event: Value) -> Value {
        state
    }

    /// Reduce a full event stream into a completed response, if terminal.
    pub fn from_events(_events: Vec<Value>) -> LlmResult<Option<Value>> {
        Err(LlmError::NotImplemented("llm response reducer"))
    }

    /// Complete a reducer state, if it saw a terminal finish.
    pub fn complete(_state: &Value) -> Option<Value> {
        None
    }

    /// Extract concatenated text from an event stream.
    pub fn text(events: &[Value]) -> String {
        events
            .iter()
            .filter(|event| event.get("type").and_then(Value::as_str) == Some("text-delta"))
            .filter_map(|event| event.get("text").and_then(Value::as_str))
            .collect()
    }
}
