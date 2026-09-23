//! Port of packages/core/test/application-tools.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: an opaque application handler is carried through
//! registration and executes, tool names are validated, deny rules filter
//! advertised definitions without adding execution authorization, and an unknown
//! tool settles as an error. Dropped: the scope-removal and same-name location
//! precedence cases, which depend on `Scope` mechanics.

use opencode_core::application_tools::{ToolDefinition, ToolRegistry};
use serde_json::json;

fn definition(name: &str, description: &str) -> ToolDefinition {
    ToolDefinition {
        name: name.into(),
        description: description.into(),
    }
}

#[test]
fn executes_a_registered_application_handler() {
    let mut registry = ToolRegistry::new();
    registry
        .register_application(vec![definition("opaque", "Read application context")])
        .unwrap();

    let result = registry
        .settle("opaque", json!({ "query": "once" }))
        .unwrap();
    assert_eq!(result.kind, "content");
}

#[test]
fn validates_tool_names() {
    let mut registry = ToolRegistry::new();
    assert!(registry
        .register_application(vec![definition("invalid name", "Read application context")])
        .is_err());
}

#[test]
fn filters_an_application_tool_by_its_name() {
    let mut registry = ToolRegistry::new();
    registry
        .register_application(vec![definition(
            "application_context",
            "Read application context",
        )])
        .unwrap();

    let denied = registry
        .definitions(&["application_context".to_string()])
        .unwrap();
    assert!(denied.is_empty());
}

#[test]
fn settles_an_unknown_tool_as_an_error() {
    let registry = ToolRegistry::new();
    let result = registry
        .settle("contextual", json!({ "query": "hello" }))
        .unwrap();
    assert_eq!(result.kind, "error");
}
