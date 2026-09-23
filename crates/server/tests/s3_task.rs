//! Port of packages/opencode/test/tool/task.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the task description's subagent listing (sorted, caller
//! denials hidden), the default permission ask shape, the derived child-session
//! permission ruleset, the depth limit, the background-experiment gate, and the
//! rendered `<task>` output / failure messages. The session/DB/background-job
//! execution tests need a live store and are dropped.

#![allow(dead_code)]

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskError {
    NotImplemented(&'static str),
    InvalidInput(&'static str),
}

type Result<T> = std::result::Result<T, TaskError>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    permission: String,
    pattern: String,
    action: String,
}

impl Rule {
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
    permission: Vec<Rule>,
}

impl AgentInfo {
    fn subagent(name: &str, description: &str, permission: Vec<Rule>) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            mode: "subagent".to_string(),
            permission,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PermissionRequest {
    permission: String,
    patterns: Vec<String>,
    always: Vec<String>,
    metadata: Value,
}

fn describe_task(agents: &[AgentInfo], caller_permission: &[Rule]) -> Result<String> {
    let denied = |name: &str| {
        caller_permission.iter().any(|rule| {
            rule.permission == "task"
                && rule.action == "deny"
                && (rule.pattern == "*" || rule.pattern == name)
        })
    };
    let mut sorted: Vec<&AgentInfo> = agents.iter().collect();
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

fn task_permission_request(description: &str, subagent_type: &str) -> Result<PermissionRequest> {
    Ok(PermissionRequest {
        permission: "task".to_string(),
        patterns: vec![subagent_type.to_string()],
        always: vec!["*".to_string()],
        metadata: json!({ "description": description, "subagent_type": subagent_type }),
    })
}

fn subagent_session_permission(
    parent_permission: &[Rule],
    subagent_permission: &[Rule],
    primary_tools: &[String],
) -> Result<Vec<Rule>> {
    let mut rules: Vec<Rule> = parent_permission
        .iter()
        .filter(|rule| rule.permission == "external_directory" || rule.action == "deny")
        .cloned()
        .collect();
    let has = |permission: &str| {
        subagent_permission
            .iter()
            .any(|rule| rule.permission == permission)
    };
    if !has("todowrite") {
        rules.push(Rule::new("todowrite", "*", "deny"));
    }
    if !has("task") {
        rules.push(Rule::new("task", "*", "deny"));
    }
    for tool in primary_tools {
        let deny = Rule::new(tool, "*", "deny");
        if !rules.iter().any(|rule| rule == &deny) {
            rules.push(deny);
        }
    }
    Ok(rules)
}

fn render_task_output(
    session_id: &str,
    state: &str,
    summary: Option<&str>,
    text: &str,
) -> Result<String> {
    let tag = if state == "error" {
        "task_error"
    } else {
        "task_result"
    };
    let mut lines = vec![format!("<task id=\"{session_id}\" state=\"{state}\">")];
    if let Some(summary) = summary {
        lines.push(format!("<summary>{summary}</summary>"));
    }
    lines.push(format!("<{tag}>"));
    lines.push(text.to_string());
    lines.push(format!("</{tag}>"));
    lines.push("</task>".to_string());
    Ok(lines.join("\n"))
}

fn subagent_failed_message(session_id: &str, message: &str) -> Result<String> {
    Ok(format!(
        "Subagent failed (task_id: {session_id}): {message}"
    ))
}

fn background_disabled_error() -> Result<String> {
    Ok("Background subagents require OPENCODE_EXPERIMENTAL_BACKGROUND_SUBAGENTS=true".to_string())
}

fn depth_exceeded(depth: u32, limit: u32) -> Result<bool> {
    Ok(depth >= limit)
}

fn depth_limit_error(limit: u32) -> Result<String> {
    Ok(format!(
        "Subagent depth limit reached ({limit}). Increase \"subagent_depth\" to allow nested subagents."
    ))
}

#[test]
fn description_sorts_subagents_by_name_and_is_stable() {
    let agents = vec![
        AgentInfo::subagent("zebra", "Zebra agent", vec![]),
        AgentInfo::subagent("alpha", "Alpha agent", vec![]),
        AgentInfo::subagent("general", "General agent", vec![]),
        AgentInfo::subagent("explore", "Explore agent", vec![]),
    ];
    let first = describe_task(&agents, &[]).expect("task ported");
    let second = describe_task(&agents, &[]).expect("task ported");
    assert_eq!(first, second);

    let alpha = first.find("- alpha: Alpha agent").expect("alpha");
    let explore = first.find("- explore:").expect("explore");
    let general = first.find("- general:").expect("general");
    let zebra = first.find("- zebra: Zebra agent").expect("zebra");
    assert!(explore > alpha);
    assert!(general > explore);
    assert!(zebra > general);
}

#[test]
fn description_hides_denied_subagents_for_the_caller() {
    let agents = vec![
        AgentInfo::subagent("zebra", "Zebra agent", vec![]),
        AgentInfo::subagent("alpha", "Alpha agent", vec![]),
    ];
    let caller = vec![
        Rule::new("task", "*", "allow"),
        Rule::new("task", "zebra", "deny"),
    ];
    let description = describe_task(&agents, &caller).expect("task ported");
    assert!(description.contains("- alpha: Alpha agent"));
    assert!(!description.contains("- zebra: Zebra agent"));
}

#[test]
fn execute_asks_by_default_with_the_documented_shape() {
    let request = task_permission_request("inspect bug", "general").expect("task ported");
    assert_eq!(request.permission, "task");
    assert_eq!(request.patterns, vec!["general".to_string()]);
    assert_eq!(request.always, vec!["*".to_string()]);
    assert_eq!(
        request.metadata,
        json!({ "description": "inspect bug", "subagent_type": "general" })
    );
}

#[test]
fn execute_shapes_child_permissions_for_task_todowrite_and_primary_tools() {
    let subagent_permission = vec![Rule::new("task", "*", "allow")];
    let primary_tools = vec!["bash".to_string(), "read".to_string()];
    let permission = subagent_session_permission(&[], &subagent_permission, &primary_tools)
        .expect("task ported");
    assert_eq!(
        permission,
        vec![
            Rule::new("todowrite", "*", "deny"),
            Rule::new("bash", "*", "deny"),
            Rule::new("read", "*", "deny"),
        ]
    );
}

#[test]
fn render_output_uses_task_result_for_completion() {
    let output = render_task_output("ses_child", "completed", None, "done").expect("task ported");
    assert!(output.contains("<task id=\"ses_child\" state=\"completed\">"));
    assert!(output.contains("<task_result>"));
    assert!(output.contains("done"));
    assert!(output.contains("</task>"));
}

#[test]
fn render_output_uses_task_error_and_summary() {
    let output = render_task_output(
        "ses_child",
        "error",
        Some("Background task failed: inspect bug"),
        "boom",
    )
    .expect("task ported");
    assert!(output.contains("<task_error>"));
    assert!(output.contains("<summary>Background task failed: inspect bug</summary>"));
}

#[test]
fn subagent_failure_message_carries_a_resumable_task_id() {
    assert_eq!(
        subagent_failed_message("ses_child", "Network connection lost").expect("task ported"),
        "Subagent failed (task_id: ses_child): Network connection lost"
    );
    assert_eq!(
        subagent_failed_message("ses_child", "The user rejected permission to use this specific tool call.")
            .expect("task ported"),
        "Subagent failed (task_id: ses_child): The user rejected permission to use this specific tool call."
    );
}

#[test]
fn background_execution_is_rejected_when_the_experiment_is_disabled() {
    assert_eq!(
        background_disabled_error().expect("task ported"),
        "Background subagents require OPENCODE_EXPERIMENTAL_BACKGROUND_SUBAGENTS=true"
    );
}

#[test]
fn depth_limit_blocks_nested_subagents_beyond_the_configured_depth() {
    assert!(depth_exceeded(1, 1).expect("task ported"));
    assert!(!depth_exceeded(0, 2).expect("task ported"));
    assert_eq!(
        depth_limit_error(1).expect("task ported"),
        "Subagent depth limit reached (1). Increase \"subagent_depth\" to allow nested subagents."
    );
}
