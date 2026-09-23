//! Port of packages/opencode/test/tool/code-mode-integration.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the catalog renderer against real MCP schemas and the
//! MCP tool-name join (`McpCatalog.toolName`). The test's live in-memory MCP
//! server + confined interpreter path is not portable without a JS engine and a
//! transport, so those execution tests are dropped.

#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeModeError {
    NotImplemented(&'static str),
}

type Result<T> = std::result::Result<T, CodeModeError>;

#[derive(Debug, Clone, PartialEq)]
struct CatalogTool {
    key: String,
    description: String,
    input_schema: Value,
    output_schema: Option<Value>,
}

impl CatalogTool {
    fn new(
        key: &str,
        description: &str,
        input_schema: Value,
        output_schema: Option<Value>,
    ) -> Self {
        Self {
            key: key.to_string(),
            description: description.to_string(),
            input_schema,
            output_schema,
        }
    }
}

fn describe_catalog(_tools: &[CatalogTool], _servers: &[String]) -> Result<String> {
    Err(CodeModeError::NotImplemented("CodeMode.describeCatalog"))
}

fn mcp_tool_name(_server: &str, _tool: &str) -> Result<String> {
    Err(CodeModeError::NotImplemented("McpCatalog.toolName"))
}

#[test]
#[ignore = "porting: code-mode catalog not implemented"]
fn appended_catalog_inlines_real_mcp_signatures() {
    let tools = vec![
        CatalogTool::new(
            "fixtures_get_text",
            "Greet someone and return the greeting as text",
            json!({ "type": "object", "properties": { "name": { "type": "string" } }, "required": ["name"] }),
            None,
        ),
        CatalogTool::new(
            "fixtures_add",
            "Add two numbers and return the structured sum",
            json!({ "type": "object", "properties": { "a": { "type": "number" }, "b": { "type": "number" } }, "required": ["a", "b"] }),
            Some(
                json!({ "type": "object", "properties": { "sum": { "type": "number" } }, "required": ["sum"] }),
            ),
        ),
        CatalogTool::new(
            "fixtures_screenshot",
            "Capture a screenshot and return it as an image",
            json!({ "type": "object", "properties": {} }),
            None,
        ),
        CatalogTool::new(
            "fixtures_boom",
            "A tool that always fails",
            json!({ "type": "object", "properties": {} }),
            None,
        ),
    ];
    let description = describe_catalog(&tools, &["fixtures".to_string()]).expect("catalog ported");

    assert!(description.contains("Available tools (COMPLETE list"));
    assert!(description.contains("- fixtures (4 tools)"));
    assert!(description.contains(
        "tools.fixtures.add(input: {\n  a: number,\n  b: number,\n}): Promise<{\n  sum: number,\n}>"
    ));
    assert!(description
        .contains("tools.fixtures.get_text(input: {\n  name: string,\n}): Promise<unknown>"));
    assert!(description.contains("// Add two numbers and return the structured sum"));
    assert!(!description.contains("$codemode"));
    assert!(description.contains("## Workflow"));
    assert!(description.contains("Do not infer or normalize tool names"));
    assert!(description.contains("bracket notation and quotes are part of the path"));
    assert!(!description.contains("total_count"));
}

#[test]
#[ignore = "porting: code-mode catalog not implemented"]
fn mcp_tool_name_joins_server_and_tool_with_underscore() {
    assert_eq!(
        mcp_tool_name("fixtures", "get_text").expect("toolName ported"),
        "fixtures_get_text"
    );
    assert_eq!(
        mcp_tool_name("weather", "current").expect("toolName ported"),
        "weather_current"
    );
}
