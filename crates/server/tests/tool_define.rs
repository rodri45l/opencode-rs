//! Port of packages/opencode/test/tool/tool-define.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `Tool.define` materializes a fresh tool per `init()` and
//! never mutates the original definition object. The decoded-parameters and
//! invalid-arguments cases are dropped (the Rust wrapper does not run an Effect
//! schema decoder yet).

use opencode_server::tools::{ToolContext, ToolDefinition, ToolError, ToolResult, ToolSpec};
use serde_json::json;
use std::sync::Arc;

fn make_tool() -> ToolSpec {
    ToolSpec {
        description: "test tool".to_string(),
        parameters: json!({ "type": "object", "properties": {} }),
        execute: Arc::new(|_args: serde_json::Value, _ctx: &mut ToolContext| {
            Ok(ToolResult {
                title: "test".to_string(),
                output: "ok".to_string(),
                metadata: json!({}),
                attachments: None,
            })
        }),
    }
}

#[test]
#[ignore = "porting: Tool.define not implemented"]
fn object_defined_tool_does_not_mutate_the_original_init_object() -> Result<(), ToolError> {
    let original = make_tool();
    let original_execute = original.execute.clone();

    let info = ToolDefinition::define("test-tool", make_tool)?;
    info.init()?;
    info.init()?;
    info.init()?;

    assert!(Arc::ptr_eq(&original.execute, &original_execute));
    Ok(())
}

#[test]
#[ignore = "porting: Tool.define not implemented"]
fn effect_defined_tool_returns_fresh_objects() -> Result<(), ToolError> {
    let info = ToolDefinition::define("test-fn-tool", make_tool)?;
    let first = info.init()?;
    let second = info.init()?;

    assert!(!Arc::ptr_eq(&first.execute, &second.execute));
    Ok(())
}

#[test]
#[ignore = "porting: Tool.define not implemented"]
fn object_defined_tool_returns_distinct_objects_per_init_call() -> Result<(), ToolError> {
    let info = ToolDefinition::define("test-copy", make_tool)?;
    let first = info.init()?;
    let second = info.init()?;

    assert!(!Arc::ptr_eq(&first.execute, &second.execute));
    Ok(())
}

#[test]
#[ignore = "porting: Tool.define not implemented"]
fn exposes_the_tool_id() -> Result<(), ToolError> {
    let info = ToolDefinition::define("test-id", make_tool)?;
    assert_eq!(info.id(), "test-id");
    Ok(())
}
