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
        Ok(vec!["*"])
    }

    /// The permission save resources.
    pub fn permission_saves() -> CoreResult<Vec<&'static str>> {
        Ok(vec!["*"])
    }

    /// The two-space JSON serialization used for the text result and content.
    pub fn format(todos: &[Todo]) -> CoreResult<String> {
        // Serialize with the field order the model-facing contract pins:
        // content, then status, then priority.
        let mut out = String::from("[");
        for (index, todo) in todos.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str("\n  {\n");
            out.push_str(&format!(
                "    \"content\": {},\n",
                serde_json::to_string(&todo.content)
                    .map_err(|error| CoreError::Message(error.to_string()))?
            ));
            out.push_str(&format!(
                "    \"status\": {},\n",
                serde_json::to_string(&todo.status)
                    .map_err(|error| CoreError::Message(error.to_string()))?
            ));
            out.push_str(&format!(
                "    \"priority\": {}\n",
                serde_json::to_string(&todo.priority)
                    .map_err(|error| CoreError::Message(error.to_string()))?
            ));
            out.push_str("  }");
        }
        if !todos.is_empty() {
            out.push('\n');
        }
        out.push(']');
        Ok(out)
    }

    /// The structured output `{ todos: [...] }`.
    pub fn structured(todos: &[Todo]) -> CoreResult<Value> {
        Ok(json!({ "todos": todos_to_value(todos) }))
    }

    /// Build the text content part.
    pub fn content(todos: &[Todo]) -> CoreResult<Value> {
        Ok(json!([{ "type": "text", "text": Self::format(todos)? }]))
    }

    /// The structured output value for the default empty list.
    pub fn empty_structured() -> Value {
        json!({ "todos": [] })
    }
}

fn todos_to_value(todos: &[Todo]) -> Value {
    Value::Array(
        todos
            .iter()
            .map(|todo| {
                json!({
                    "content": todo.content,
                    "status": todo.status,
                    "priority": todo.priority,
                })
            })
            .collect(),
    )
}
