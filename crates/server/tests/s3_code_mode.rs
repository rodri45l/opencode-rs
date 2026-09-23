//! Port of packages/opencode/test/tool/code-mode.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the `execute` (Code Mode) parameter contract, the pure
//! catalog/instruction renderer (`describeCatalog`), hard-deny permission
//! visibility, and the MCP-result projection that strips media into
//! attachments. The confined JS interpreter itself is not ported here (it needs
//! a JavaScript engine), so the execution-path tests are dropped.

#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeModeError {
    NotImplemented(&'static str),
    InvalidParameters(&'static str),
    InvalidInput(&'static str),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionRule {
    permission: String,
    pattern: String,
    action: String,
}

impl PermissionRule {
    fn deny(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
            pattern: "*".to_string(),
            action: "deny".to_string(),
        }
    }

    fn ask(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
            pattern: "*".to_string(),
            action: "ask".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum McpBlock {
    Text(String),
    Image {
        data: String,
        mime: String,
    },
    Resource {
        uri: String,
        mime: Option<String>,
        blob: String,
    },
    ResourceLink {
        uri: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Projected {
    value: Value,
    attachments: Vec<Value>,
}

fn base_description() -> &'static str {
    "Run a confined orchestration script with access to connected MCP tools."
}

fn parameters_json_schema() -> Result<Value> {
    Ok(json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "description": "Script body executed by the confined interpreter."
            }
        },
        "required": ["code"]
    }))
}

fn validate_parameters(input: &Value) -> Result<Value> {
    match input.get("code").and_then(Value::as_str) {
        Some(_) => Ok(input.clone()),
        None => Err(CodeModeError::InvalidParameters("code is required")),
    }
}

fn visible_tools(tools: &[CatalogTool], permission: &[PermissionRule]) -> Result<Vec<CatalogTool>> {
    let denied = |key: &str| {
        permission.iter().any(|rule| {
            rule.action == "deny"
                && rule.permission == key
                && (rule.pattern == "*" || rule.pattern == key)
        })
    };
    Ok(tools
        .iter()
        .filter(|tool| !denied(&tool.key))
        .cloned()
        .collect())
}

#[derive(Debug, Clone)]
struct ToolDescription {
    path: String,
    description: String,
    signature: String,
}

fn path_expression(path: &str) -> String {
    let mut out = String::from("tools");
    for segment in path.split('.') {
        let ident = !segment.is_empty()
            && segment
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
                .unwrap_or(false)
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
        if ident {
            out.push('.');
            out.push_str(segment);
        } else {
            out.push('[');
            out.push_str(&serde_json::to_string(segment).unwrap_or_default());
            out.push(']');
        }
    }
    out
}

fn render_ts(schema: Option<&Value>, pretty: bool, depth: usize) -> String {
    let Some(schema) = schema else {
        return "unknown".to_string();
    };
    if let Some(any) = schema.get("anyOf").and_then(Value::as_array) {
        let members: Vec<String> = any
            .iter()
            .map(|member| render_ts(Some(member), pretty, depth + 1))
            .collect();
        return members.join(" | ");
    }
    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        return types
            .iter()
            .map(|t| render_ts(Some(&json!({ "type": t })), pretty, depth + 1))
            .collect::<Vec<_>>()
            .join(" | ");
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "string".to_string(),
        Some("number") | Some("integer") => "number".to_string(),
        Some("boolean") => "boolean".to_string(),
        Some("null") => "null".to_string(),
        Some("array") => format!(
            "Array<{}>",
            render_ts(schema.get("items"), pretty, depth + 1)
        ),
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
            if !pretty {
                let fields: Vec<String> = properties
                    .iter()
                    .map(|(name, value)| {
                        let optional = if required.contains(&name.as_str()) {
                            ""
                        } else {
                            "?"
                        };
                        format!(
                            "{name}{optional}: {}",
                            render_ts(Some(value), pretty, depth + 1)
                        )
                    })
                    .collect();
                return format!("{{ {} }}", fields.join("; "));
            }
            let pad = "  ".repeat(depth + 1);
            let mut lines = Vec::new();
            for (name, value) in properties {
                if let Some(description) = value.get("description").and_then(Value::as_str) {
                    lines.push(format!("{pad}/** {description} */"));
                }
                let optional = if required.contains(&name.as_str()) {
                    ""
                } else {
                    "?"
                };
                lines.push(format!(
                    "{pad}{name}{optional}: {},",
                    render_ts(Some(value), pretty, depth + 1)
                ));
            }
            format!("{{\n{}\n{}}}", lines.join("\n"), "  ".repeat(depth))
        }
    }
}

fn group_tools(tools: &[CatalogTool], servers: &[String]) -> Vec<(String, Vec<ToolDescription>)> {
    let mut sorted_servers: Vec<&String> = servers.iter().collect();
    sorted_servers.sort_by_key(|server| std::cmp::Reverse(server.len()));
    let mut keys: Vec<&CatalogTool> = tools.iter().collect();
    keys.sort_by(|a, b| a.key.cmp(&b.key));
    let mut groups: Vec<(String, Vec<ToolDescription>)> = Vec::new();
    for tool in keys {
        let server = sorted_servers
            .iter()
            .find(|name| tool.key.starts_with(&format!("{name}_")))
            .map(|name| name.to_string())
            .unwrap_or_else(|| tool.key.split('_').next().unwrap_or(&tool.key).to_string());
        let local = tool
            .key
            .strip_prefix(&format!("{server}_"))
            .unwrap_or(&tool.key)
            .to_string();
        let path = format!("{server}.{local}");
        let signature = format!(
            "{}(input: {}): Promise<{}>",
            path_expression(&path),
            render_ts(Some(&tool.input_schema), true, 0),
            render_ts(tool.output_schema.as_ref(), true, 0)
        );
        let description = ToolDescription {
            path,
            description: tool.description.clone(),
            signature,
        };
        match groups.iter_mut().find(|(name, _)| name == &server) {
            Some((_, group)) => group.push(description),
            None => groups.push((server, vec![description])),
        }
    }
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    groups
}

fn catalog_line(tool: &ToolDescription) -> String {
    let line = tool.description.split('\n').next().unwrap_or("").trim();
    let description = if line.chars().count() > 120 {
        format!("{}...", line.chars().take(119).collect::<String>())
    } else {
        line.to_string()
    };
    if description.is_empty() {
        format!("  - {}", tool.signature)
    } else {
        format!("  - {} // {}", tool.signature, description)
    }
}

fn catalog_cost(tool: &ToolDescription) -> i64 {
    (catalog_line(tool).len() as f64 / 4.0).round() as i64
}

fn search_signature() -> &'static str {
    "tools.$codemode.search(input: {\n  query?: string,\n  namespace?: string,\n  limit?: number,\n  offset?: number,\n}): Promise<{\n  items: Array<{\n      path: string,\n      description: string,\n      signature: string,\n    }>,\n  remaining: number,\n  next: {\n      offset: number,\n    } | null,\n}>"
}

fn describe_catalog(tools: &[CatalogTool], servers: &[String]) -> Result<String> {
    let groups = group_tools(tools, servers);
    let described: usize = groups.iter().map(|(_, group)| group.len()).sum();

    let budget: i64 = 2000;
    #[allow(clippy::type_complexity)]
    let mut selections: Vec<(String, Vec<ToolDescription>, Vec<bool>, Vec<usize>)> = groups
        .into_iter()
        .map(|(namespace, group)| {
            let mut queue: Vec<usize> = (0..group.len()).collect();
            queue.sort_by(|&a, &b| {
                catalog_cost(&group[a])
                    .cmp(&catalog_cost(&group[b]))
                    .then(group[a].path.cmp(&group[b].path))
            });
            let picked = vec![false; group.len()];
            (namespace, group, picked, queue)
        })
        .collect();

    let mut used: i64 = 0;
    let mut active: Vec<usize> = (0..selections.len())
        .filter(|&i| !selections[i].3.is_empty())
        .collect();
    while !active.is_empty() {
        let mut still: Vec<usize> = Vec::new();
        for &i in &active {
            let next = selections[i].3[0];
            let cost = catalog_cost(&selections[i].1[next]);
            if used + cost > budget {
                continue;
            }
            selections[i].3.remove(0);
            selections[i].2[next] = true;
            used += cost;
            if !selections[i].3.is_empty() {
                still.push(i);
            }
        }
        active = still;
    }
    let total_shown: usize = selections
        .iter()
        .map(|(_, _, picked, _)| picked.iter().filter(|p| **p).count())
        .sum();
    let complete = total_shown == described;
    let empty = described == 0;

    let mut lines: Vec<String> = Vec::new();
    lines.push(if empty {
        "This is a restricted JavaScript language for calling tools, not a general-purpose runtime.".to_string()
    } else if complete {
        "This is a restricted JavaScript language for calling tools, not a general-purpose runtime. Inside the confined interpreter, `tools` contains the Code Mode tools listed below and internal runtime tools; surrounding agent tools are not available.".to_string()
    } else {
        "This is a restricted JavaScript language for calling tools, not a general-purpose runtime. Inside the confined interpreter, `tools` contains the Code Mode tools listed or searchable below and internal runtime tools; surrounding agent tools are not available.".to_string()
    });
    if !empty {
        lines.push("Do not infer or normalize tool names; use only exact signatures shown below or returned by search.".to_string());
    }

    if !empty {
        lines.push(String::new());
        lines.push("## Workflow".to_string());
        lines.push(String::new());
        if complete {
            lines.push("1. Pick a tool from the list under `## Available tools` - each line is the exact call signature; use it as-is rather than guessing segments.".to_string());
            lines.push("2. Call it using the exact signature shown: `const result = await tools.<namespace>.<tool>(input)`; bracket notation and quotes are part of the path.".to_string());
            lines.push("3. Return only the fields you need from structured results; narrow unknown results before reading fields, and avoid returning large raw payloads.".to_string());
        } else {
            lines.push("1. If needed, discover tools: `return await tools.$codemode.search({ query: \"<intent + key nouns>\" })`.".to_string());
            lines.push("2. In the next execution, copy a returned path exactly, call it, and return only the needed fields.".to_string());
        }

        lines.push(String::new());
        lines.push("## Rules".to_string());
        lines.push(String::new());
        lines.push(if complete {
            "- Only Code Mode tools listed here and internal runtime tools are available; surrounding agent tools are not implicitly exposed.".to_string()
        } else {
            "- Only Code Mode tools listed here or returned by `tools.$codemode.search` and internal runtime tools are available; surrounding agent tools are not implicitly exposed.".to_string()
        });
        lines.push("- Filter, aggregate, and transform collections in code - never return them raw or call a tool per item across messages.".to_string());
        lines.push("- A result typed `Promise<unknown>` may be structured data or text. Before reading fields, check that it is a non-null object and not an array; otherwise handle the returned text or primitive directly.".to_string());
        lines.push("- Run independent calls in parallel: `await Promise.all(items.map((item) => tools.<namespace>.<tool>(item)))`, or use `tools.<namespace>[\"tool-name\"](item)` when the listed signature uses bracket notation.".to_string());
        lines.push("- `Object.keys(tools)` lists namespaces; `Object.keys(tools.<namespace>)` lists its tools; `for...in` works on both.".to_string());
        if !complete {
            lines.push("- Browse one namespace: `await tools.$codemode.search({ query: \"\", namespace: \"<name>\" })`.".to_string());
            lines.push(
                "- If search returns `next`, repeat the same search with `offset: next.offset`."
                    .to_string(),
            );
        }

        lines.push(String::new());
        lines.push("## Language".to_string());
        lines.push(String::new());
        lines.push("Use common JavaScript data operations, functions, control flow, selected standard-library methods, and awaited tool calls. Built-ins include Date, RegExp, Map, Set, URL, URLSearchParams, and URI encoding helpers.".to_string());
        lines.push("Modules/imports, classes, generators, timers, fetch, eval, prototype access, unlisted methods, and promise chaining are unavailable. Use Code Mode tools for external operations. Use await with try/catch.".to_string());
        lines.push("Dates and URLs serialize to strings at data boundaries; Map/Set/RegExp/URLSearchParams serialize to `{}`.".to_string());
    }

    lines.push(String::new());
    if empty {
        lines.push("## Available tools".to_string());
        lines.push(String::new());
        lines.push("No tools are currently available.".to_string());
    } else {
        if complete {
            lines.push("## Available tools (COMPLETE list - every tool is shown below with its full call signature)".to_string());
        } else {
            lines.push(format!(
                "## Available tools (PARTIAL - {total_shown} of {described} shown; find the rest with tools.$codemode.search)"
            ));
        }
        lines.push(String::new());
        for (namespace, group, picked, _) in &selections {
            let count = group.len();
            let noun = if count == 1 { "tool" } else { "tools" };
            let shown = picked.iter().filter(|p| **p).count();
            let label = if shown == count {
                format!("{count} {noun}")
            } else if shown == 0 {
                format!("{count} {noun}, none shown")
            } else {
                format!("{count} {noun}, {shown} shown")
            };
            lines.push(format!("- {namespace} ({label})"));
            for (index, tool) in group.iter().enumerate() {
                if picked[index] {
                    lines.push(catalog_line(tool));
                }
            }
        }
        if !complete {
            lines.push(String::new());
            lines.push("Search returns complete callable signatures:".to_string());
            lines.push(format!("- {}", search_signature()));
        }
    }

    Ok(lines.join("\n"))
}

fn describe_for(
    tools: &[CatalogTool],
    servers: &[String],
    permission: &[PermissionRule],
) -> Result<String> {
    let visible = visible_tools(tools, permission)?;
    describe_catalog(&visible, servers)
}

fn last_segment(uri: &str) -> Option<String> {
    let trimmed = uri
        .split(['?', '#'])
        .next()
        .unwrap_or(uri)
        .trim_end_matches('/');
    let segment = trimmed.rsplit('/').next().unwrap_or("");
    if segment.is_empty() {
        None
    } else {
        Some(segment.to_string())
    }
}

fn project_mcp_result(blocks: &[McpBlock], structured: Option<Value>) -> Result<Projected> {
    if let Some(value) = structured {
        return Ok(Projected {
            value,
            attachments: Vec::new(),
        });
    }
    let mut text: Vec<String> = Vec::new();
    let mut attachments: Vec<Value> = Vec::new();
    let mut files = 0usize;
    let mut images = 0usize;
    #[allow(unused_mut)]
    let mut push = |attachments: &mut Vec<Value>,
                    files: &mut usize,
                    images: &mut usize,
                    mime: &str,
                    url: String,
                    filename: Option<&str>| {
        *files += 1;
        if mime.starts_with("image/") {
            *images += 1;
        }
        attachments.push(attachment(mime, &url, filename));
    };
    for block in blocks {
        match block {
            McpBlock::Text(value) => text.push(value.clone()),
            McpBlock::Image { data, mime } => {
                let url = format!("data:{mime};base64,{data}");
                push(&mut attachments, &mut files, &mut images, mime, url, None);
            }
            McpBlock::Resource { uri, mime, blob } => {
                let mime = mime
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".to_string());
                let url = format!("data:{mime};base64,{blob}");
                let filename = last_segment(uri);
                push(
                    &mut attachments,
                    &mut files,
                    &mut images,
                    &mime,
                    url,
                    filename.as_deref(),
                );
            }
            McpBlock::ResourceLink { uri, name } => {
                text.push(format!("{name}: {uri}"));
            }
        }
    }
    let value = if !text.is_empty() {
        Value::String(text.join("\n"))
    } else if files > 0 {
        let noun = if files == images { "image" } else { "file" };
        Value::String(format!(
            "[{files} {noun}{} attached to the result]",
            if files == 1 { "" } else { "s" }
        ))
    } else {
        Value::Null
    };
    Ok(Projected { value, attachments })
}

fn metadata_input(input: &Value) -> Result<Option<Value>> {
    match input {
        Value::Object(map) if map.is_empty() => Ok(None),
        Value::Object(_) => Ok(Some(input.clone())),
        _ => Ok(Some(json!({ "input": input.clone() }))),
    }
}

fn tool(key: &str, description: &str) -> CatalogTool {
    CatalogTool::new(
        key,
        description,
        json!({ "type": "object", "properties": {} }),
        None,
    )
}

fn attachment(mime: &str, url: &str, filename: Option<&str>) -> Value {
    match filename {
        Some(name) => json!({ "type": "file", "mime": mime, "url": url, "filename": name }),
        None => json!({ "type": "file", "mime": mime, "url": url }),
    }
}

#[test]
fn parameters_require_code_with_description() {
    assert_eq!(
        validate_parameters(&json!({ "code": "return 1" })).expect("code-mode ported"),
        json!({ "code": "return 1" })
    );
    assert!(matches!(
        validate_parameters(&json!({})),
        Err(CodeModeError::InvalidParameters(_))
    ));
    let schema = parameters_json_schema().expect("code-mode ported");
    assert_eq!(
        schema["properties"]["code"]["description"],
        json!("Script body executed by the confined interpreter.")
    );
}

#[test]
fn groups_multi_underscore_server_by_longest_prefix() {
    let description = describe_for(
        &[tool("my_server_do_thing", "do thing")],
        &["my_server".to_string()],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains("- my_server (1 tool)"));
    assert!(description.contains("tools.my_server.do_thing("));
}

#[test]
fn group_by_server_uses_whole_key_when_no_underscore() {
    let description =
        describe_for(&[tool("standalone", "standalone")], &[], &[]).expect("code-mode ported");
    assert!(description.contains("- standalone (1 tool)"));
    assert!(description.contains("tools.standalone.standalone("));
}

#[test]
fn catalog_carries_raw_mcp_schemas() {
    let description = describe_for(
        &[CatalogTool::new(
            "weather_current",
            "current",
            json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
            Some(json!({ "type": "object", "properties": { "tempC": { "type": "number" } }, "required": ["tempC"] })),
        )],
        &["weather".to_string()],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains(
        "tools.weather.current(input: {\n  city: string,\n}): Promise<{\n  tempC: number,\n}>"
    ));
}

#[test]
fn static_base_description_carries_no_catalog() {
    assert_eq!(
        base_description(),
        "Run a confined orchestration script with access to connected MCP tools."
    );
    assert!(!base_description().contains("Available tools"));
    assert!(!base_description().contains("list_issues"));
}

#[test]
fn small_catalog_inlines_every_full_signature() {
    let tools = vec![
        CatalogTool::new(
            "github_create_issue",
            "create_issue",
            json!({
                "type": "object",
                "properties": { "title": { "type": "string" }, "body": { "type": "string" } },
                "required": ["title"]
            }),
            None,
        ),
        tool("github_list_issues", "list_issues"),
        tool("linear_search", "search"),
    ];
    let description = describe_for(&tools, &["github".to_string(), "linear".to_string()], &[])
        .expect("code-mode ported");

    assert!(description.contains("Available tools (COMPLETE list"));
    assert!(description.contains("- github (2 tools)"));
    assert!(description.contains("- linear (1 tool)"));
    assert!(description.contains(
        "tools.github.create_issue(input: {\n  title: string,\n  body?: string,\n}): Promise<unknown>"
    ));
    assert!(description.contains("tools.github.list_issues("));
    assert!(description.contains("tools.linear.search("));
    assert!(description.contains("tools.linear.search(input: {}): Promise<unknown>"));
    assert!(!description.contains("$codemode"));
    assert!(!description.contains("Browse one namespace"));
    assert!(description.contains("## Workflow"));
    assert!(description.contains("1. Pick a tool from the list under `## Available tools`"));
    assert!(!description.contains("JSON.parse(res)"));
    assert!(description.contains("check that it is a non-null object and not an array"));
    assert!(description.contains("Return only the fields you need"));
    assert!(!description.contains("total_count"));
}

#[test]
fn signatures_render_declared_output_schema_as_return_type() {
    let description = describe_for(
        &[CatalogTool::new(
            "weather_current",
            "current",
            json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
            Some(json!({
                "type": "object",
                "properties": { "tempC": { "type": "number" }, "summary": { "type": "string" } },
                "required": ["tempC"]
            })),
        )],
        &[],
        &[],
    )
    .expect("code-mode ported");
    assert!(description.contains(
        "tools.weather.current(input: {\n  city: string,\n}): Promise<{\n  tempC: number,\n  summary?: string,\n}>"
    ));
}

#[test]
fn large_catalogs_inline_budgeted_partial_list_plus_search() {
    let mut tools = Vec::new();
    let filler =
        "a searchable description of this operation that consumes catalog budget ".repeat(3);
    for i in 0..150 {
        tools.push(CatalogTool::new(
            &format!("alpha_op_{i}"),
            &format!("{filler}{i}"),
            json!({ "type": "object", "properties": { "value": { "type": "string" }, "count": { "type": "number" } } }),
            None,
        ));
    }
    tools.push(CatalogTool::new(
        "zeta_only_tool",
        "only_tool",
        json!({
            "type": "object",
            "properties": { "topic": { "type": "string", "description": "Subject to look up" } },
            "required": ["topic"]
        }),
        None,
    ));
    let description = describe_for(&tools, &["alpha".to_string(), "zeta".to_string()], &[])
        .expect("code-mode ported");

    assert!(description.contains("Available tools (PARTIAL - "));
    assert!(description.contains("- alpha (150 tools, "));
    assert!(description.contains(" shown)"));
    assert!(description.contains("- zeta (1 tool)\n"));
    assert!(description.contains(
        "tools.zeta.only_tool(input: {\n  /** Subject to look up */\n  topic: string,\n}): Promise<unknown>"
    ));
    assert!(description.contains("tools.$codemode.search("));
    assert!(description.contains("  limit?: number,\n  offset?: number,"));
    assert!(description.contains("  remaining: number,\n  next: {"));
    assert!(description.contains("      offset: number,\n    } | null,"));
    assert!(description.contains(
        "1. If needed, discover tools: `return await tools.$codemode.search({ query: \"<intent + key nouns>\" })`."
    ));
    assert!(description.contains(
        "- Browse one namespace: `await tools.$codemode.search({ query: \"\", namespace: \"<name>\" })`."
    ));
    assert!(!description.contains("total_count"));
    assert!(description.contains("tools.alpha.op_0("));
    assert!(!description.contains("tools.alpha.op_99("));
}

#[test]
fn hard_denied_tool_never_enters_the_catalog() {
    let tools = vec![
        tool("github_create_issue", "create_issue"),
        tool("github_list_issues", "list_issues"),
    ];
    let description = describe_for(
        &tools,
        &["github".to_string()],
        &[PermissionRule::deny("github_create_issue")],
    )
    .expect("code-mode ported");
    assert!(description.contains("tools.github.list_issues("));
    assert!(!description.contains("create_issue"));
    assert!(description.contains("- github (1 tool)"));
}

#[test]
fn ask_level_tool_stays_fully_visible() {
    let tools = vec![
        tool("github_create_issue", "create_issue"),
        tool("github_list_issues", "list_issues"),
    ];
    let description = describe_for(
        &tools,
        &["github".to_string()],
        &[PermissionRule::ask("github_create_issue")],
    )
    .expect("code-mode ported");
    assert!(description.contains("tools.github.create_issue("));
    assert!(description.contains("tools.github.list_issues("));
    assert!(description.contains("- github (2 tools)"));
}

#[test]
fn visible_tools_hides_only_hard_denies() {
    let tools = vec![
        tool("a_tool", "a"),
        tool("b_tool", "b"),
        tool("c_tool", "c"),
    ];
    let mut c_deny = PermissionRule::deny("c_tool");
    c_deny.pattern = "something".to_string();
    let visible = visible_tools(
        &tools,
        &[
            PermissionRule::deny("a_tool"),
            PermissionRule::ask("b_tool"),
            c_deny,
        ],
    )
    .expect("code-mode ported");
    let keys: Vec<String> = visible.into_iter().map(|tool| tool.key).collect();
    assert_eq!(keys, vec!["b_tool".to_string(), "c_tool".to_string()]);
}

#[test]
fn structured_content_is_exposed_natively() {
    let projected = project_mcp_result(
        &[McpBlock::Text("ignored".to_string())],
        Some(json!({ "sum": 3 })),
    )
    .expect("code-mode ported");
    assert_eq!(projected.value, json!({ "sum": 3 }));
    assert!(projected.attachments.is_empty());
}

#[test]
fn media_only_result_returns_a_marker() {
    let projected = project_mcp_result(
        &[McpBlock::Image {
            data: "PNGDATA".to_string(),
            mime: "image/png".to_string(),
        }],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(projected.value, json!("[1 image attached to the result]"));
    assert_eq!(
        projected.attachments,
        vec![attachment(
            "image/png",
            "data:image/png;base64,PNGDATA",
            None
        )]
    );
}

#[test]
fn media_only_markers_distinguish_images_from_mixed_files() {
    let images = project_mcp_result(
        &[
            McpBlock::Image {
                data: "PNG1".to_string(),
                mime: "image/png".to_string(),
            },
            McpBlock::Image {
                data: "PNG2".to_string(),
                mime: "image/png".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(images.value, json!("[2 images attached to the result]"));

    let mixed = project_mcp_result(
        &[
            McpBlock::Image {
                data: "PNG3".to_string(),
                mime: "image/png".to_string(),
            },
            McpBlock::Resource {
                uri: "file:///tmp/report.pdf".to_string(),
                mime: Some("application/pdf".to_string()),
                blob: "PDF1".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(mixed.value, json!("[2 files attached to the result]"));
    assert_eq!(
        mixed.attachments,
        vec![
            attachment("image/png", "data:image/png;base64,PNG3", None),
            attachment(
                "application/pdf",
                "data:application/pdf;base64,PDF1",
                Some("report.pdf")
            ),
        ]
    );
}

#[test]
fn resource_links_flow_to_the_program_as_text() {
    let projected = project_mcp_result(
        &[
            McpBlock::ResourceLink {
                uri: "https://example.com/guide.pdf".to_string(),
                name: "guide.pdf".to_string(),
            },
            McpBlock::ResourceLink {
                uri: "file:///tmp/notes.md".to_string(),
                name: "notes.md".to_string(),
            },
        ],
        None,
    )
    .expect("code-mode ported");
    assert_eq!(
        projected.value,
        json!("guide.pdf: https://example.com/guide.pdf\nnotes.md: file:///tmp/notes.md")
    );
    assert!(projected.attachments.is_empty());
}

#[test]
fn empty_object_input_is_omitted_from_call_metadata() {
    assert_eq!(metadata_input(&json!({})).expect("code-mode ported"), None);
    assert_eq!(
        metadata_input(&json!({ "name": "Ada" })).expect("code-mode ported"),
        Some(json!({ "name": "Ada" }))
    );
    assert_eq!(
        metadata_input(&json!([1, 2])).expect("code-mode ported"),
        Some(json!({ "input": [1, 2] }))
    );
}
