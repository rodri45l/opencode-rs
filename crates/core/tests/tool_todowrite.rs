//! Port of packages/core/test/tool-todowrite.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool registers as `todowrite`, asserts the wildcard
//! resources for action `todowrite`, serializes the todo list as two-space JSON
//! for both the text result and text content, exposes the structured
//! `{ todos }` output, and reports `Unable to update todos` when denied.
//! Re-derived: the `Database`/`SessionTodo`/`ToolRegistry`/`Permission` service
//! wiring and persistence are dropped.

use opencode_core::tool_todowrite::{Todo, TodoWriteTool};
use serde_json::json;

const NOTE: &str = "porting: todo-write tool not implemented";

fn todos() -> Vec<Todo> {
    vec![Todo {
        content: "Implement slice".into(),
        status: "in_progress".into(),
        priority: "high".into(),
    }]
}

#[test]
#[ignore = "porting: todo-write tool not implemented"]
fn registers_with_the_wildcard_permission_resources() {
    assert_eq!(TodoWriteTool::NAME, "todowrite");
    assert_eq!(TodoWriteTool::ACTION, "todowrite");
    assert_eq!(
        TodoWriteTool::permission_resources().expect(NOTE),
        vec!["*"]
    );
    assert_eq!(TodoWriteTool::permission_saves().expect(NOTE), vec!["*"]);
}

#[test]
#[ignore = "porting: todo-write tool not implemented"]
fn serializes_todos_as_two_space_json_for_the_text_result() {
    let expected = "[\n  {\n    \"content\": \"Implement slice\",\n    \"status\": \"in_progress\",\n    \"priority\": \"high\"\n  }\n]";
    assert_eq!(TodoWriteTool::format(&todos()).expect(NOTE), expected);
    assert_eq!(
        TodoWriteTool::content(&todos()).expect(NOTE),
        json!([{ "type": "text", "text": expected }])
    );
    assert_eq!(
        TodoWriteTool::structured(&todos()).expect(NOTE),
        json!({ "todos": [ { "content": "Implement slice", "status": "in_progress", "priority": "high" } ] })
    );
}

#[test]
#[ignore = "porting: todo-write tool not implemented"]
fn reports_the_denied_output() {
    assert_eq!(TodoWriteTool::DENIED, "Unable to update todos");
}
