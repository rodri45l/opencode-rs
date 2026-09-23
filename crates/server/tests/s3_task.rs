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

fn describe_task(_agents: &[AgentInfo], _caller_permission: &[Rule]) -> Result<String> {
    Err(TaskError::NotImplemented("task subagent description"))
}

fn task_permission_request(_description: &str, _subagent_type: &str) -> Result<PermissionRequest> {
    Err(TaskError::NotImplemented("task permission request"))
}

fn subagent_session_permission(
    _parent_permission: &[Rule],
    _subagent_permission: &[Rule],
    _primary_tools: &[String],
) -> Result<Vec<Rule>> {
    Err(TaskError::NotImplemented("deriveSubagentSessionPermission"))
}

fn render_task_output(
    _session_id: &str,
    _state: &str,
    _summary: Option<&str>,
    _text: &str,
) -> Result<String> {
    Err(TaskError::NotImplemented("task output renderer"))
}

fn subagent_failed_message(_session_id: &str, _message: &str) -> Result<String> {
    Err(TaskError::NotImplemented("subagent failure message"))
}

fn background_disabled_error() -> Result<String> {
    Err(TaskError::NotImplemented("background subagent gate"))
}

fn depth_exceeded(_depth: u32, _limit: u32) -> Result<bool> {
    Err(TaskError::NotImplemented("subagent depth check"))
}

fn depth_limit_error(_limit: u32) -> Result<String> {
    Err(TaskError::NotImplemented("subagent depth error"))
}

#[test]
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
fn render_output_uses_task_result_for_completion() {
    let output = render_task_output("ses_child", "completed", None, "done").expect("task ported");
    assert!(output.contains("<task id=\"ses_child\" state=\"completed\">"));
    assert!(output.contains("<task_result>"));
    assert!(output.contains("done"));
    assert!(output.contains("</task>"));
}

#[test]
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
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
#[ignore = "porting: task tool not implemented"]
fn background_execution_is_rejected_when_the_experiment_is_disabled() {
    assert_eq!(
        background_disabled_error().expect("task ported"),
        "Background subagents require OPENCODE_EXPERIMENTAL_BACKGROUND_SUBAGENTS=true"
    );
}

#[test]
#[ignore = "porting: task tool not implemented"]
fn depth_limit_blocks_nested_subagents_beyond_the_configured_depth() {
    assert!(depth_exceeded(1, 1).expect("task ported"));
    assert!(!depth_exceeded(0, 2).expect("task ported"));
    assert_eq!(
        depth_limit_error(1).expect("task ported"),
        "Subagent depth limit reached (1). Increase \"subagent_depth\" to allow nested subagents."
    );
}
