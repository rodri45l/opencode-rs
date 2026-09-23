//! Streamed tool-call assembly shared by protocol adapters.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// Incremental tool-call state machine.
pub struct ToolStream;

impl ToolStream {
    /// Empty tool state.
    pub fn empty() -> Value {
        serde_json::json!({})
    }

    /// Start tracking a tool call.
    pub fn start(state: Value, item: Value, tool: Value) -> Value {
        serde_json::json!({ "state": state, "item": item, "tool": tool })
    }

    /// Append a delta or start a new tool call.
    pub fn append_or_start(
        _state: Value,
        _index: usize,
        _delta: Value,
        _missing: &str,
    ) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool stream appendOrStart"))
    }

    /// Append a delta to an existing tool call.
    pub fn append_existing(
        _state: Value,
        _index: usize,
        _text: &str,
        _missing: &str,
    ) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool stream appendExisting"))
    }

    /// Finish a tool call by parsing accumulated input.
    pub fn finish(_state: Value, _index: usize) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool stream finish"))
    }

    /// Finish a tool call with an explicit input override.
    pub fn finish_with_input(_state: Value, _item: Value, _input: &str) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool stream finishWithInput"))
    }

    /// Finish every tracked tool call.
    pub fn finish_all(_state: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool stream finishAll"))
    }

    /// Whether a result is a `NotImplemented` error sentinel.
    pub fn is_error(value: &Value) -> bool {
        value.get("error").is_some()
    }
}
