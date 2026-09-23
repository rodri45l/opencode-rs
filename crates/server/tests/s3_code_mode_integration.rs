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

fn render_ts(schema: Option<&Value>, depth: usize) -> String {
    let Some(schema) = schema else {
        return "unknown".to_string();
    };
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "string".to_string(),
        Some("number") | Some("integer") => "number".to_string(),
        Some("boolean") => "boolean".to_string(),
        Some("array") => format!("Array<{}>", render_ts(schema.get("items"), depth + 1)),
        _ => {
            let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
                return "unknown".to_string();
            };
            let required: Vec<&str> = schema
                .get("required")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if properties.is_empty() {
                return "{}".to_string();
            }
            let pad = "  ".repeat(depth + 1);
            let lines: Vec<String> = properties
                .iter()
                .map(|(name, value)| {
                    let optional = if required.contains(&name.as_str()) {
                        ""
                    } else {
                        "?"
                    };
                    format!(
                        "{pad}{name}{optional}: {},",
                        render_ts(Some(value), depth + 1)
                    )
                })
                .collect();
            format!("{{\n{}\n{}}}", lines.join("\n"), "  ".repeat(depth))
        }
    }
}

fn describe_catalog(tools: &[CatalogTool], servers: &[String]) -> Result<String> {
    let mut sorted_servers: Vec<&String> = servers.iter().collect();
    sorted_servers.sort_by_key(|server| std::cmp::Reverse(server.len()));
    let mut keys: Vec<&CatalogTool> = tools.iter().collect();
    keys.sort_by(|a, b| a.key.cmp(&b.key));

    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for tool in keys {
        let server = sorted_servers
            .iter()
            .find(|name| tool.key.starts_with(&format!("{name}_")))
            .map(|name| name.to_string())
            .unwrap_or_else(|| tool.key.split('_').next().unwrap_or(&tool.key).to_string());
        let local = tool
            .key
            .strip_prefix(&format!("{server}_"))
            .unwrap_or(&tool.key);
        let signature = format!(
            "tools.{server}.{local}(input: {}): Promise<{}>",
            render_ts(Some(&tool.input_schema), 0),
            render_ts(tool.output_schema.as_ref(), 0)
        );
        let line = format!("  - {signature} // {}", tool.description);
        match groups.iter_mut().find(|(name, _)| name == &server) {
            Some((_, group)) => group.push(line),
            None => groups.push((server, vec![line])),
        }
    }
    groups.sort_by(|a, b| a.0.cmp(&b.0));

    let mut lines = vec![
        "This is a restricted JavaScript language for calling tools, not a general-purpose runtime. Inside the confined interpreter, `tools` contains the Code Mode tools listed below and internal runtime tools; surrounding agent tools are not available.".to_string(),
        "Do not infer or normalize tool names; use only exact signatures shown below or returned by search.".to_string(),
        String::new(),
        "## Workflow".to_string(),
        String::new(),
        "1. Pick a tool from the list under `## Available tools` - each line is the exact call signature; use it as-is rather than guessing segments.".to_string(),
        "2. Call it using the exact signature shown: `const result = await tools.<namespace>.<tool>(input)`; bracket notation and quotes are part of the path.".to_string(),
        "3. Return only the fields you need from structured results; narrow unknown results before reading fields, and avoid returning large raw payloads.".to_string(),
        String::new(),
        "## Rules".to_string(),
        String::new(),
        "- A result typed `Promise<unknown>` may be structured data or text. Before reading fields, check that it is a non-null object and not an array; otherwise handle the returned text or primitive directly.".to_string(),
        String::new(),
        "## Available tools (COMPLETE list - every tool is shown below with its full call signature)".to_string(),
        String::new(),
    ];
    for (namespace, group) in &groups {
        let count = group.len();
        let noun = if count == 1 { "tool" } else { "tools" };
        lines.push(format!("- {namespace} ({count} {noun})"));
        lines.extend(group.iter().cloned());
    }
    Ok(lines.join("\n"))
}

fn mcp_tool_name(server: &str, tool: &str) -> Result<String> {
    Ok(format!("{server}_{tool}"))
}

#[test]
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
