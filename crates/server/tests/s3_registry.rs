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

fn registry_ids(
    _code_mode_enabled: bool,
    _code_mode_has_tools: bool,
    _plugin_tools: &[String],
) -> Result<Vec<String>> {
    Err(RegistryError::NotImplemented("ToolRegistry.ids"))
}

fn code_mode_execute_description(_tools: &[CatalogTool], _servers: &[String]) -> Result<String> {
    Err(RegistryError::NotImplemented(
        "code-mode catalog description",
    ))
}

fn task_tool_schema(_background_subagents: bool) -> Result<Value> {
    Err(RegistryError::NotImplemented("task tool json schema"))
}

fn custom_tool_wire_schema(_args: Option<&Value>) -> Result<Value> {
    Err(RegistryError::NotImplemented("custom tool wire schema"))
}

fn file_tool_ids(
    _filename: &str,
    _has_default_export: bool,
    _named_exports: &[String],
) -> Result<Vec<String>> {
    Err(RegistryError::NotImplemented("custom tool file scan"))
}

fn validate_tool_input(_schema: &Value, _input: &Value) -> Result<Value> {
    Err(RegistryError::NotImplemented(
        "custom tool input validation",
    ))
}

fn describe_task(_agents: &[AgentInfo], _caller_permission: &[PermissionRule]) -> Result<String> {
    Err(RegistryError::NotImplemented("ToolRegistry.describeTask"))
}

fn has_background_parameter(schema: &Value) -> bool {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.contains_key("background"))
        .unwrap_or(false)
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn does_not_expose_task_status() {
    let ids = registry_ids(false, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "task_status"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn does_not_expose_execute_unless_code_mode_is_enabled() {
    let ids = registry_ids(false, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "execute"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
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
#[ignore = "porting: tool registry not implemented"]
fn does_not_expose_execute_when_code_mode_has_no_visible_tools() {
    let ids = registry_ids(true, false, &[]).expect("registry ported");
    assert!(!ids.iter().any(|id| id == "execute"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn hides_task_background_parameter_unless_experiment_enabled() {
    let schema = task_tool_schema(false).expect("registry ported");
    assert!(!has_background_parameter(&schema));
    let enabled = task_tool_schema(true).expect("registry ported");
    assert!(has_background_parameter(&enabled));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn loads_tools_from_singular_tool_directory() {
    let ids = file_tool_ids("hello.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "hello"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn ignores_non_tool_exports() {
    let ids = file_tool_ids("mixed.ts", true, &["helper".to_string()]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "mixed"));
    assert!(!ids.iter().any(|id| id == "mixed_helper"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn tolerates_custom_tool_with_undefined_args() {
    let schema = custom_tool_wire_schema(None).expect("registry ported");
    assert_eq!(schema, json!({ "type": "object", "properties": {} }));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn tolerates_plugin_tool_registered_with_undefined_args() {
    let ids =
        registry_ids(false, false, &["broken_plugin_tool".to_string()]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "read"));
    assert!(ids.iter().any(|id| id == "broken_plugin_tool"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
fn loads_tools_from_plural_tools_directory() {
    let ids = file_tool_ids("hello.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "hello"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
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
#[ignore = "porting: tool registry not implemented"]
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
#[ignore = "porting: tool registry not implemented"]
fn loads_tool_with_external_dependencies_without_crashing() {
    let ids = file_tool_ids("cowsay.ts", true, &[]).expect("registry ported");
    assert!(ids.iter().any(|id| id == "cowsay"));
}

#[test]
#[ignore = "porting: tool registry not implemented"]
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
#[ignore = "porting: tool registry not implemented"]
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
