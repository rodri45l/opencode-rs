//! Tool authoring and the tool runtime.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// Tool authoring.
pub struct Tool;

impl Tool {
    /// Build a tool from a description and input schema.
    pub fn make(input: Value) -> Value {
        input
    }

    /// Execute a tool with raw input.
    pub fn execute(_tool: Value, _input: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool execute"))
    }

    /// Project tools into protocol tool definitions.
    pub fn to_definitions(_tools: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("toDefinitions"))
    }
}

/// Canonical tool content.
pub struct ToolContent;

impl ToolContent {
    /// Validate and normalise a content part.
    pub fn decode(_input: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool content decode"))
    }
}

/// Canonical tool output.
pub struct ToolOutput;

impl ToolOutput {
    /// Build a tool output from structured and content parts.
    pub fn make(structured: Value, content: Value) -> Value {
        serde_json::json!({ "structured": structured, "content": content })
    }

    /// Convert an output into a result value.
    pub fn to_result_value(_output: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool output to result value"))
    }

    /// Convert a result value back into an output.
    pub fn from_result_value(_value: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool output from result value"))
    }
}

/// A tool execution failure surfaced to the model.
#[derive(Debug, Clone)]
pub struct ToolFailure {
    /// Human-readable message.
    pub message: String,
}

/// Context handed to a tool's `execute`.
#[derive(Debug, Clone)]
pub struct ToolExecuteContext {
    /// Tool call id.
    pub id: String,
    /// Tool name.
    pub name: String,
}

/// The tool runtime dispatcher.
pub struct ToolRuntime;

impl ToolRuntime {
    /// Dispatch one tool call and collect its result events.
    pub fn dispatch(_tools: Value, _call: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool runtime dispatch"))
    }

    /// Run the model/tool loop and stream events.
    pub fn run(_input: Value) -> LlmResult<Vec<Value>> {
        Err(LlmError::NotImplemented("tool runtime run"))
    }
}
