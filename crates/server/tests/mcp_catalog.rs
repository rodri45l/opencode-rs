//! Port of packages/opencode/test/mcp/catalog.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `convertTool` preserves content when structured content is
//! present, and synthesizes a text block only when content is empty. The
//! paginated discovery case drives the MCP SDK client and is dropped.

use opencode_server::port::mcp::{convert_tool_result, McpToolCallResult};
use serde_json::json;

#[test]
fn preserves_content_when_structured_content_is_also_present() {
    let content = vec![json!({ "type": "image", "mimeType": "image/png", "data": "AAAA" })];
    let structured = json!({ "image": { "mimeType": "image/png", "data": "AAAA" } });

    let output = convert_tool_result(McpToolCallResult {
        content: content.clone(),
        structured_content: Some(structured.clone()),
    });

    assert_eq!(output.content, content);
    assert_eq!(output.structured_content, Some(structured));
}

#[test]
fn falls_back_to_structured_content_only_when_content_is_absent() {
    let structured = json!({ "results": [{ "title": "one" }] });

    let output = convert_tool_result(McpToolCallResult {
        content: vec![],
        structured_content: Some(structured.clone()),
    });

    assert_eq!(
        output.content,
        vec![json!({
            "type": "text",
            "text": serde_json::to_string(&structured).expect("structured content serializes"),
        })]
    );
    assert_eq!(output.structured_content, Some(structured));
}
