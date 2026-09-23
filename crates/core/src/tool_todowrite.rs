//! Todo-write tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/todowrite.ts`: the
//! tool is registered as `todo_write` (upstream name `todowrite`), asserts the
//! wildcard permission resources for action `todowrite`, serializes the todo
//! list as two-space JSON for both the text result and text content, exposes the
//! structured `{ todos }` output, and reports `Unable to update todos` when
//! denied. The `Database`/`SessionTodo`/`ToolRegistry` wiring is dropped; the
//! pure formatting and permission surface remain.

use serde_json::{json, Value};

use crate::{CoreError, CoreResult};

/// One todo entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    /// The todo content.
    pub content: String,
    /// The status (`pending`, `in_progress`, `completed`, ...).
    pub status: String,
    /// The priority (`high`, `medium`, `low`).
    pub priority: String,
}

/// The todo-write tool.
#[derive(Debug, Default)]
pub struct TodoWriteTool;

impl TodoWriteTool {
    /// The tool name.
    pub const NAME: &'static str = "todowrite";

    /// The permission action.
    pub const ACTION: &'static str = "todowrite";

    /// The error output when the update is denied.
    pub const DENIED: &'static str = "Unable to update todos";

    /// The permission resources.
    pub fn permission_resources() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_todowrite::TodoWriteTool::permission_resources",
        ))
    }

    /// The permission save resources.
    pub fn permission_saves() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_todowrite::TodoWriteTool::permission_saves",
        ))
    }

    /// The two-space JSON serialization used for the text result and content.
    pub fn format(_todos: &[Todo]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_todowrite::TodoWriteTool::format",
        ))
    }

    /// The structured output `{ todos: [...] }`.
    pub fn structured(_todos: &[Todo]) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "tool_todowrite::TodoWriteTool::structured",
        ))
    }

    /// Build the text content part.
    pub fn content(_todos: &[Todo]) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "tool_todowrite::TodoWriteTool::content",
        ))
    }

    /// The structured output value for the default empty list.
    pub fn empty_structured() -> Value {
        json!({ "todos": [] })
    }
}
