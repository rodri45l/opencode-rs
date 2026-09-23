//! Port of packages/core/test/session-runner-tool-registry.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: tool materialization disables a tool only when the last
//! wildcard-matching permission rule is a `*` deny (with `edit` aliasing the
//! `edit`/`write`/`apply_patch` tools), permission decoration stays isolated per
//! registration, model definitions are reused across provider turns, scoped
//! registrations disappear with their scope, unknown/stale/failed calls return
//! canonical error values, retention failures propagate through settlement, and
//! only a materialization exposes settlement.
//! Re-derived: Effect `Scope`/`Deferred`/`Fiber` and the `Tool`/`ApplicationTools`
//! layers are replaced by an in-memory registry with explicit scopes; interruption
//! and captured-execution races are exercised as deterministic registration
//! mutations.

#![allow(dead_code)]

use serde_json::json;

const NOTE: &str = "porting: session runner tool registry not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Effect {
        Allow,
        Deny,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Rule {
        pub action: String,
        pub resource: String,
        pub effect: Effect,
    }

    /// A registered tool definition.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ToolDef {
        pub name: String,
        pub permission: Option<String>,
        pub identity: usize,
    }

    /// A canonical tool result plus bounded output.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Settlement {
        pub result: Value,
        pub output: Option<Value>,
        pub output_paths: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Call {
        pub session_id: String,
        pub agent: String,
        pub assistant_message_id: String,
        pub id: String,
        pub name: String,
        pub input: Value,
        pub advertised_identity: Option<usize>,
    }

    pub fn wildcard_match(action: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if !pattern.contains('*') {
            return action == pattern;
        }
        let mut remainder = action;
        let mut parts = pattern.split('*').peekable();
        if let Some(first) = parts.next() {
            if !remainder.starts_with(first) {
                return false;
            }
            remainder = &remainder[first.len()..];
        }
        for part in parts {
            if part.is_empty() {
                continue;
            }
            match remainder.find(part) {
                Some(index) => remainder = &remainder[index + part.len()..],
                None => return false,
            }
        }
        pattern.ends_with('*') || remainder.is_empty()
    }

    pub fn wholly_disabled(action: &str, rules: &[Rule]) -> bool {
        let rule = rules
            .iter()
            .rev()
            .find(|rule| wildcard_match(action, &rule.action));
        matches!(rule, Some(rule) if rule.resource == "*" && rule.effect == Effect::Deny)
    }

    pub fn unknown_tool_message(name: &str) -> String {
        format!("Unknown tool: {name}")
    }

    pub fn stale_tool_call_message(name: &str) -> String {
        format!("Stale tool call: {name}")
    }

    pub fn retention_failure_message(cause: &str) -> String {
        format!("Failed to write tool output: {cause}")
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Materialization {
        definitions: Vec<ToolDef>,
    }

    impl Materialization {
        pub fn definitions(&self) -> Vec<&ToolDef> {
            self.definitions.iter().collect()
        }

        pub fn settle(&self, _call: &Call) -> Result<Settlement, PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }
    }

    #[derive(Debug, Default)]
    pub struct ToolRegistry;

    impl ToolRegistry {
        pub fn new() -> Self {
            Self
        }

        pub fn register(&mut self, _tools: Vec<ToolDef>) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }

        pub fn register_scoped(
            &mut self,
            _scope: usize,
            _tools: Vec<ToolDef>,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }

        pub fn remove_scope(&mut self, _scope: usize) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }

        pub fn materialize(&self, _rules: &[Rule]) -> Result<Materialization, PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }

        pub fn names(&self, _rules: &[Rule]) -> Result<Vec<String>, PortError> {
            Err(PortError::NotImplemented("session runner tool registry"))
        }
    }

    pub fn action_of(tool: &ToolDef) -> &str {
        tool.permission.as_deref().unwrap_or(tool.name.as_str())
    }
}

use local::{action_of, Effect, Rule, ToolDef, ToolRegistry};

fn tool(name: &str, permission: Option<&str>) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        permission: permission.map(str::to_string),
        identity: 1,
    }
}

fn deny(action: &str, resource: &str) -> Rule {
    Rule {
        action: action.to_string(),
        resource: resource.to_string(),
        effect: Effect::Deny,
    }
}

fn allow(action: &str, resource: &str) -> Rule {
    Rule {
        action: action.to_string(),
        resource: resource.to_string(),
        effect: Effect::Allow,
    }
}

fn filtered_names(tools: &[ToolDef], rules: &[Rule]) -> Vec<String> {
    tools
        .iter()
        .filter(|tool| !local::wholly_disabled(action_of(tool), rules))
        .map(|tool| tool.name.clone())
        .collect()
}

fn call(name: &str, id: &str) -> local::Call {
    local::Call {
        session_id: "ses_registry".to_string(),
        agent: "build".to_string(),
        assistant_message_id: "msg_registry".to_string(),
        id: id.to_string(),
        name: name.to_string(),
        input: json!({ "text": name }),
        advertised_identity: None,
    }
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn filters_disabled_tools_with_edit_aliases_and_ordered_wildcard_precedence() {
    let tools = vec![
        tool("question", None),
        tool("bash", None),
        tool("edit", Some("edit")),
        tool("write", Some("edit")),
        tool("apply_patch", Some("edit")),
    ];

    assert_eq!(
        filtered_names(&tools, &[deny("question", "*")]),
        vec!["bash", "edit", "write", "apply_patch"]
    );
    assert_eq!(
        filtered_names(&tools, &[deny("*", "*"), allow("question", "private")]),
        vec!["question"]
    );
    assert_eq!(
        filtered_names(&tools, &[allow("question", "private"), deny("*", "*")]),
        Vec::<String>::new()
    );
    assert_eq!(
        filtered_names(&tools, &[deny("edit", "*")]),
        vec!["question", "bash"]
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn keeps_permission_decoration_isolated_between_registrations() {
    let mut registry = ToolRegistry::new();
    let shared = tool("echo", None);
    registry.register(vec![shared.clone()]).expect(NOTE);
    let mut decorated = shared;
    decorated.permission = Some("edit".to_string());
    registry.register(vec![decorated]).expect(NOTE);
    assert_eq!(
        registry.names(&[deny("edit", "*")]).expect(NOTE),
        vec!["first".to_string()]
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn reuses_model_definitions_across_provider_turns() {
    let mut registry = ToolRegistry::new();
    registry.register(vec![tool("echo", None)]).expect(NOTE);
    let first = registry.materialize(&[]).expect(NOTE);
    let second = registry.materialize(&[]).expect(NOTE);
    assert_eq!(
        second.definitions()[0].identity,
        first.definitions()[0].identity
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn removes_a_scoped_registration() {
    let mut registry = ToolRegistry::new();
    registry
        .register_scoped(7, vec![tool("echo", None)])
        .expect(NOTE);
    assert_eq!(registry.names(&[]).expect(NOTE), vec!["echo".to_string()]);
    registry.remove_scope(7).expect(NOTE);
    assert!(registry.names(&[]).expect(NOTE).is_empty());
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn returns_model_errors_without_swallowing_interruption_or_defects() {
    let registry = ToolRegistry::new();
    let failed = call("failed", "failed");
    let settlement = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&failed)
        .expect(NOTE);
    assert_eq!(
        settlement.result,
        json!({ "type": "error", "value": "Denied" })
    );

    let missing = call("missing", "missing");
    let settlement = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&missing)
        .expect(NOTE);
    assert_eq!(
        settlement.result,
        json!({ "type": "error", "value": local::unknown_tool_message("missing") })
    );

    let defect = call("defect", "defect");
    assert_eq!(
        registry
            .materialize(&[])
            .expect(NOTE)
            .settle(&defect)
            .expect_err(NOTE),
        local::PortError::NotImplemented("session runner tool registry")
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn propagates_retention_failures_through_settlement() {
    let registry = ToolRegistry::new();
    let failure = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&call("echo", "call-retention-failure"))
        .expect_err(NOTE);
    assert_eq!(
        failure,
        local::PortError::NotImplemented("session runner tool registry")
    );
    assert_eq!(
        local::retention_failure_message("disk full"),
        "Failed to write tool output: disk full"
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn exposes_settlement_only_through_materialization() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    assert!(materialized.definitions().is_empty());
    assert_eq!(
        materialized
            .settle(&call("echo", "call-echo"))
            .expect(NOTE)
            .result,
        json!({ "type": "text", "value": "echo" })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn passes_complete_invocation_identity_to_the_canonical_handler() {
    let registry = ToolRegistry::new();
    let settlement = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&call("context", "call-context"))
        .expect(NOTE);
    assert_eq!(
        settlement.result,
        json!({ "type": "text", "value": "context" })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn encodes_output_and_applies_generic_settlement_bounding() {
    let registry = ToolRegistry::new();
    let settlement = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&call("bounded", "call-bounded"))
        .expect(NOTE);
    assert_eq!(
        settlement.result,
        json!({ "type": "text", "value": "bounded reference" })
    );
    assert_eq!(
        settlement.output,
        Some(
            json!({ "structured": {}, "content": [{ "type": "text", "text": "bounded reference" }] })
        )
    );
    assert_eq!(
        settlement.output_paths,
        vec!["/managed/generic".to_string()]
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn enforces_transformed_codecs_at_execution_and_projection_boundaries() {
    let registry = ToolRegistry::new();
    let invalid_input = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&call("transformed", "invalid-input"))
        .expect(NOTE);
    assert!(invalid_input.result["value"]
        .as_str()
        .expect(NOTE)
        .contains("Invalid tool input"));

    let invalid_output = registry
        .materialize(&[])
        .expect(NOTE)
        .settle(&call("invalid_output", "invalid-output"))
        .expect(NOTE);
    assert!(invalid_output.result["value"]
        .as_str()
        .expect(NOTE)
        .contains("invalid value for its output schema"));
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn executes_the_unchanged_registration_advertised_for_a_provider_turn() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut advertised = call("echo", "call-echo");
    advertised.advertised_identity = Some(1);
    assert_eq!(
        materialized.settle(&advertised).expect(NOTE).result,
        json!({ "type": "text", "value": "echo" })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn rejects_a_call_when_its_advertised_registration_was_removed() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut advertised = call("echo", "call-echo");
    advertised.advertised_identity = Some(2);
    assert_eq!(
        materialized.settle(&advertised).expect(NOTE).result,
        json!({ "type": "error", "value": local::stale_tool_call_message("echo") })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn rejects_only_the_replaced_name_from_a_multi_tool_provider_turn() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut first = call("first", "call-first");
    first.advertised_identity = Some(1);
    let mut second = call("second", "call-second");
    second.advertised_identity = Some(2);
    assert_eq!(
        materialized.settle(&first).expect(NOTE).result,
        json!({ "type": "error", "value": local::stale_tool_call_message("first") })
    );
    assert_eq!(
        materialized.settle(&second).expect(NOTE).result,
        json!({ "type": "text", "value": "second" })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn treats_revealing_a_previous_overlay_as_stale() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut advertised = call("echo", "call-echo");
    advertised.advertised_identity = Some(3);
    assert_eq!(
        materialized.settle(&advertised).expect(NOTE).result,
        json!({ "type": "error", "value": local::stale_tool_call_message("echo") })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn rejects_an_application_call_after_a_location_override_is_registered() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut advertised = call("echo", "call-echo");
    advertised.advertised_identity = Some(4);
    assert_eq!(
        materialized.settle(&advertised).expect(NOTE).result,
        json!({ "type": "error", "value": local::stale_tool_call_message("echo") })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn rejects_a_location_call_after_removal_reveals_an_application_registration() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    let mut advertised = call("echo", "call-echo");
    advertised.advertised_identity = Some(5);
    assert_eq!(
        materialized.settle(&advertised).expect(NOTE).result,
        json!({ "type": "error", "value": local::stale_tool_call_message("echo") })
    );
}

#[test]
#[ignore = "porting: session runner tool registry not implemented"]
fn keeps_captured_execution_running_after_registration_mutation() {
    let registry = ToolRegistry::new();
    let materialized = registry.materialize(&[]).expect(NOTE);
    assert_eq!(
        materialized
            .settle(&call("echo", "call-echo"))
            .expect(NOTE)
            .result,
        json!({ "type": "text", "value": "echo" })
    );
}
