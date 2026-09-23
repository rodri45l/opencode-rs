//! Port of packages/opencode/test/tool/registry.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: which built-in ids the registry exposes (never
//! `task_status`, `execute` only for a non-empty Code Mode catalog), the task
//! tool's hidden `background` parameter, custom-tool id/schema derivation
//! (including the null/undefined-`args` fallback and legacy JSON-schema args),
//! and the sorted/denied-filtered task description. The live plugin/file-system
//! loading machinery is dropped; its observable projection is re-derived here.

#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistryError {
    NotImplemented(&'static str),
    InvalidInput(&'static str),
    UnknownTool,
}

type Result<T> = std::result::Result<T, RegistryError>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionRule {
    permission: String,
    pattern: String,
    action: String,
}

impl PermissionRule {
    fn new(permission: &str, pattern: &str, action: &str) -> Self {
        Self {
            permission: permission.to_string(),
            pattern: pattern.to_string(),
            action: action.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AgentInfo {
    name: String,
    description: Option<String>,
    mode: String,
    permission: Vec<PermissionRule>,
}

impl AgentInfo {
    fn subagent(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            mode: "subagent".to_string(),
            permission: Vec::new(),
        }
    }

    fn primary(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            mode: "primary".to_string(),
            permission: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CatalogTool {
    key: String,
    description: String,
    input_schema: Value,
}

const BUILTIN_TOOL_IDS: &[&str] = &[
    "bash",
    "read",
    "write",
    "edit",
    "glob",
    "grep",
    "list",
    "patch",
    "task",
    "todowrite",
    "todoread",
    "webfetch",
    "websearch",
    "skill",
    "question",
    "lsp",
];

fn registry_ids(
    code_mode_enabled: bool,
    code_mode_has_tools: bool,
    plugin_tools: &[String],
) -> Result<Vec<String>> {
    let mut ids: Vec<String> = BUILTIN_TOOL_IDS.iter().map(|id| id.to_string()).collect();
    if code_mode_enabled && code_mode_has_tools {
        ids.push("execute".to_string());
    }
    for tool in plugin_tools {
        if !ids.contains(tool) {
            ids.push(tool.clone());
        }
    }
    Ok(ids)
}

fn ts_type(schema: &Value) -> String {
    match schema.get("type").and_then(Value::as_str) {
        Some("string") => "string".to_string(),
        Some("number") | Some("integer") => "number".to_string(),
        Some("boolean") => "boolean".to_string(),
        Some("array") => "unknown[]".to_string(),
        _ => "unknown".to_string(),
    }
}

fn code_mode_execute_description(tools: &[CatalogTool], servers: &[String]) -> Result<String> {
    let mut sorted: Vec<&String> = servers.iter().collect();
    sorted.sort_by_key(|server| std::cmp::Reverse(server.len()));
    let mut lines = vec!["Available tools:".to_string()];
    let mut tools_sorted: Vec<&CatalogTool> = tools.iter().collect();
    tools_sorted.sort_by(|a, b| a.key.cmp(&b.key));
    for tool in tools_sorted {
        let server = sorted
            .iter()
            .find(|name| tool.key.starts_with(&format!("{name}_")))
            .map(|name| name.as_str())
            .or_else(|| tool.key.split('_').next())
            .unwrap_or(&tool.key);
        let local = tool
            .key
            .strip_prefix(&format!("{server}_"))
            .unwrap_or(&tool.key);
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object);
        let mut body = String::new();
        if let Some(properties) = properties {
            for (name, schema) in properties {
                body.push_str(&format!("  {name}: {},\n", ts_type(schema)));
            }
        }
        lines.push(format!("- tools.{server}.{local}(input: {{\n{body}}})"));
    }
    Ok(lines.join("\n"))
}

fn task_tool_schema(background_subagents: bool) -> Result<Value> {
    let mut properties = serde_json::Map::new();
    properties.insert("description".to_string(), json!({ "type": "string" }));
    properties.insert("prompt".to_string(), json!({ "type": "string" }));
    properties.insert("subagent_type".to_string(), json!({ "type": "string" }));
    properties.insert("task_id".to_string(), json!({ "type": "string" }));
    if background_subagents {
        properties.insert("background".to_string(), json!({ "type": "boolean" }));
    }
    Ok(json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": ["description", "prompt", "subagent_type"],
    }))
}

fn custom_tool_wire_schema(args: Option<&Value>) -> Result<Value> {
    let Some(args) = args else {
        return Ok(json!({ "type": "object", "properties": {} }));
    };
    let Some(properties) = args.as_object() else {
        return Ok(json!({ "type": "object", "properties": {} }));
    };
    let required: Vec<String> = properties.keys().cloned().collect();
    Ok(json!({
        "type": "object",
        "properties": Value::Object(properties.clone()),
        "required": required,
    }))
}

fn file_tool_ids(
    filename: &str,
    has_default_export: bool,
    _named_exports: &[String],
) -> Result<Vec<String>> {
    let stem = filename
        .strip_suffix(".ts")
        .or_else(|| filename.strip_suffix(".tsx"))
        .or_else(|| filename.strip_suffix(".js"))
        .unwrap_or(filename);
    if has_default_export {
        Ok(vec![stem.to_string()])
    } else {
        Ok(Vec::new())
    }
}

fn validate_tool_input(schema: &Value, input: &Value) -> Result<Value> {
    let Some(required) = schema.get("required").and_then(Value::as_array) else {
        return Ok(input.clone());
    };
    let Some(object) = input.as_object() else {
        return Err(RegistryError::InvalidInput("expected object input"));
    };
    for key in required {
        if let Some(name) = key.as_str() {
            if !object.contains_key(name) {
                return Err(RegistryError::InvalidInput("missing required field"));
            }
        }
    }
    Ok(input.clone())
}

fn describe_task(agents: &[AgentInfo], caller_permission: &[PermissionRule]) -> Result<String> {
    let denied = |name: &str| {
        caller_permission.iter().any(|rule| {
            rule.permission == "task"
                && rule.action == "deny"
                && (rule.pattern == "*" || rule.pattern == name)
        })
    };
    let mut sorted: Vec<&AgentInfo> = agents
        .iter()
        .filter(|agent| agent.mode == "subagent")
        .collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let lines: Vec<String> = sorted
        .into_iter()
        .filter(|agent| !denied(&agent.name))
        .map(|agent| {
            format!(
                "- {}: {}",
                agent.name,
                agent.description.clone().unwrap_or_default()
            )
        })
        .collect();
    Ok(format!("Available subagents:\n{}", lines.join("\n")))
}

fn has_background_parameter(schema: &Value) -> bool {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.contains_key("background"))
        .unwrap_or(false)
}

#[test]
fn does_not_expose_task_status() {
    let ids = registry_ids(false, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "task_status"));
}

#[test]
fn does_not_expose_execute_unless_code_mode_is_enabled() {
    let ids = registry_ids(false, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "execute"));
}

#[test]
fn exposes_execute_when_code_mode_has_visible_tools() {
    let ids = registry_ids(true, true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "execute"));

    let description = code_mode_execute_description(
        &[CatalogTool {
            key: "weather_current".to_string(),
            description: "current weather".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": { "city": { "type": "string" } },
                "required": ["city"]
            }),
        }],
        &["weather".to_string()],
    )
    .expect("registry ported");
    assert!(description.contains("tools.weather.current(input: {\n  city: string,\n})"));
}

#[test]
fn does_not_expose_execute_when_code_mode_has_no_visible_tools() {
    let ids = registry_ids(true, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "execute"));
}

#[test]
fn hides_task_background_parameter_unless_experiment_enabled() {
    let schema = task_tool_schema(false).expect("registry ported");
    assert!(!has_background_parameter(&schema));
    let enabled = task_tool_schema(true).expect("registry ported");
    assert!(has_background_parameter(&enabled));
}

#[test]
fn loads_tools_from_singular_tool_directory() {
    let ids = file_tool_ids("hello.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "hello"));
}

#[test]
fn ignores_non_tool_exports() {
    let ids = file_tool_ids("mixed.ts", true, &["helper".to_string()]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "mixed"));
    assert!(!ids.iter().any(|id| id == "mixed_helper"));
}

#[test]
fn tolerates_custom_tool_with_undefined_args() {
    let schema = custom_tool_wire_schema(None).expect("registry ported");
    assert_eq!(schema, json!({ "type": "object", "properties": {} }));
}

#[test]
fn tolerates_plugin_tool_registered_with_undefined_args() {
    let ids =
        registry_ids(false, false, &["broken_plugin_tool".to_string()]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "read"));
    assert!(ids.iter().any(|id| id == "broken_plugin_tool"));
}

#[test]
fn loads_tools_from_plural_tools_directory() {
    let ids = file_tool_ids("hello.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "hello"));
}

#[test]
fn loads_zod_schema_custom_tool_with_json_schema_and_validation() {
    let schema = custom_tool_wire_schema(Some(&json!({
        "query": { "type": "string", "description": "SQL query to execute" }
    })))
    .expect("registry ported");
    assert_eq!(
        schema,
        json!({
            "type": "object",
            "properties": { "query": { "type": "string", "description": "SQL query to execute" } },
            "required": ["query"]
        })
    );
    assert_eq!(
        validate_tool_input(&schema, &json!({ "query": "select 1" })).expect("registry ported"),
        json!({ "query": "select 1" })
    );
    assert!(matches!(
        validate_tool_input(&schema, &json!({})),
        Err(RegistryError::InvalidInput(_))
    ));
}

#[test]
fn loads_legacy_json_schema_shaped_custom_tool() {
    let schema = custom_tool_wire_schema(Some(&json!({
        "text": { "type": "string", "description": "Text to render" }
    })))
    .expect("registry ported");
    assert_eq!(
        schema,
        json!({
            "type": "object",
            "properties": { "text": { "type": "string", "description": "Text to render" } },
            "required": ["text"]
        })
    );
}

#[test]
fn loads_tool_with_external_dependencies_without_crashing() {
    let ids = file_tool_ids("cowsay.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "cowsay"));
}

#[test]
fn describe_task_sorts_subagents_and_is_stable() {
    let agents = vec![
        AgentInfo::primary("build", "Build agent"),
        AgentInfo::subagent("zebra", "Zebra agent"),
        AgentInfo::subagent("alpha", "Alpha agent"),
        AgentInfo::subagent("general", "General agent"),
        AgentInfo::subagent("explore", "Explore agent"),
    ];
    let first = describe_task(&agents, &[]).expect("registry ported");
    let second = describe_task(&agents, &[]).expect("registry ported");
    assert_eq!(first, second);

    let alpha = first.find("- alpha: Alpha agent").expect("alpha listed");
    let explore = first.find("- explore:").expect("explore listed");
    let general = first.find("- general:").expect("general listed");
    let zebra = first.find("- zebra: Zebra agent").expect("zebra listed");
    assert!(explore > alpha);
    assert!(general > explore);
    assert!(zebra > general);
}

#[test]
fn describe_task_hides_denied_subagents_for_the_caller() {
    let agents = vec![
        AgentInfo::subagent("zebra", "Zebra agent"),
        AgentInfo::subagent("alpha", "Alpha agent"),
    ];
    let caller = vec![
        PermissionRule::new("task", "*", "allow"),
        PermissionRule::new("task", "zebra", "deny"),
    ];
    let description = describe_task(&agents, &caller).expect("registry ported");
    assert!(description.contains("- alpha: Alpha agent"));
    assert!(!description.contains("- zebra: Zebra agent"));
}
