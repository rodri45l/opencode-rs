//! Code-mode interpreter and tool runtime.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.
//!
//! The reference package executes a restricted JavaScript program against a
//! host tool tree. That interpreter is not ported yet, so the API surface is
//! defined here and every execution returns a typed [`CodeModeError`].

pub mod tool_schema;

use serde_json::Value;

/// The category of a code-mode failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeModeErrorKind {
    ToolFailure,
    InvalidToolOutput,
    InvalidToolInput,
    UnknownTool,
    ToolCallLimitExceeded,
    TimeoutExceeded,
    UnsupportedSyntax,
    InvalidDataValue,
    InvalidConfiguration,
}

/// A typed code-mode failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeModeError {
    pub kind: CodeModeErrorKind,
    pub message: String,
}

impl std::fmt::Display for CodeModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for CodeModeError {}

/// A tool call observed during an execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub name: String,
}

/// The result of a code-mode execution.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeModeResult {
    pub ok: bool,
    pub value: Option<Value>,
    pub logs: Vec<String>,
    pub tool_calls: Vec<ToolCall>,
    pub truncated: Option<bool>,
    pub error: Option<CodeModeError>,
}

/// Execution budgets. Absent fields mean unlimited.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub max_output_bytes: Option<usize>,
    pub timeout_ms: Option<u64>,
    pub max_tool_calls: Option<i64>,
}

fn not_implemented(topic: &'static str) -> CodeModeError {
    CodeModeError {
        kind: CodeModeErrorKind::UnsupportedSyntax,
        message: format!("not implemented: {topic}"),
    }
}

/// Entry point for one-shot and reusable code-mode execution.
pub struct CodeMode;

impl CodeMode {
    /// Execute a program with no explicit limits.
    pub fn execute(_code: &str) -> Result<CodeModeResult, CodeModeError> {
        Err(not_implemented("code-mode interpreter"))
    }

    /// Execute a program with explicit limits.
    pub fn execute_with_limits(
        _code: &str,
        _limits: &ExecutionLimits,
    ) -> Result<CodeModeResult, CodeModeError> {
        Err(not_implemented("code-mode interpreter"))
    }
}

/// Helpers that mirror the reference tool-runtime boundary.
pub struct ToolRuntime;

impl ToolRuntime {
    /// Copy a value out of the sandbox, normalizing non-finite numbers to null.
    pub fn copy_out(value: &Value) -> Value {
        match value {
            Value::Number(number) => {
                if number
                    .as_f64()
                    .map(|value| value.is_finite())
                    .unwrap_or(true)
                {
                    value.clone()
                } else {
                    Value::Null
                }
            }
            Value::Array(items) => Value::Array(items.iter().map(ToolRuntime::copy_out).collect()),
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(key, child)| (key.clone(), ToolRuntime::copy_out(child)))
                    .collect(),
            ),
            other => other.clone(),
        }
    }
}
