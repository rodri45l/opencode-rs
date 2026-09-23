//! Port of packages/opencode/test/agent/plan-mode-subagent-bypass.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a subagent's own permissions take precedence over its
//! parent agent's restrictions, while parent session denies remain hard
//! runtime ceilings. The reference `testAgent` helper is inlined.

use opencode_server::agent::{
    derive_subagent_session_permission, AgentCatalog, AgentInfo, AgentMode,
};
use opencode_server::permission::{disabled, evaluate, from_config, merge, Action, Ruleset};
use serde_json::json;
use std::collections::BTreeSet;

fn test_agent(name: &str, mode: AgentMode, permission: serde_json::Value) -> AgentInfo {
    AgentInfo {
        name: name.to_string(),
        mode,
        description: None,
        permission: from_config(&permission),
        options: serde_json::Value::Null,
    }
}

#[test]
#[ignore = "porting: agent.subagent-permissions not implemented"]
fn subagent_permissions_take_precedence_over_parent_agent_restrictions() {
    let catalog = AgentCatalog::new();
    let plan_agent = catalog.get("plan").expect("plan agent");
    let general_agent = catalog.get("general").expect("general agent");

    assert_eq!(
        evaluate("edit", "/some/file.ts", &plan_agent.permission).action,
        Action::Deny
    );

    let parent_session_permission = Ruleset::new();
    let subagent_session_permission =
        derive_subagent_session_permission(&parent_session_permission, &general_agent);
    let effective = merge(&general_agent.permission, &subagent_session_permission);

    assert_ne!(
        evaluate("edit", "/some/file.ts", &effective).action,
        Action::Deny
    );
    assert_eq!(
        disabled(&["edit", "write", "apply_patch"], &effective),
        BTreeSet::new()
    );
}

#[test]
#[ignore = "porting: agent.subagent-permissions not implemented"]
fn subagents_own_read_only_restriction_remains_effective() {
    let catalog = AgentCatalog::new();
    let explore = catalog.get("explore").expect("explore agent");

    let parent_session_permission = Ruleset::new();
    let subagent_session_permission =
        derive_subagent_session_permission(&parent_session_permission, &explore);
    let effective = merge(&explore.permission, &subagent_session_permission);

    assert_eq!(evaluate("edit", "/x.ts", &effective).action, Action::Deny);
}

#[test]
#[ignore = "porting: agent.subagent-permissions not implemented"]
fn custom_subagent_can_explicitly_enable_edits_denied_to_its_parent_agent() {
    let catalog = AgentCatalog::with_config(json!({
        "agent": {
            "my_subagent": {
                "description": "A user-defined subagent",
                "mode": "subagent",
                "permission": { "edit": "allow" }
            }
        }
    }));
    let plan_agent = catalog.get("plan").expect("plan agent");
    let my = catalog.get("my_subagent").expect("custom subagent");

    let parent_session_permission = Ruleset::new();
    let subagent_session_permission =
        derive_subagent_session_permission(&parent_session_permission, &my);
    let effective = merge(&my.permission, &subagent_session_permission);

    assert_eq!(
        evaluate("edit", "/some/file.ts", &plan_agent.permission).action,
        Action::Deny
    );
    assert_eq!(
        evaluate("edit", "/some/file.ts", &effective).action,
        Action::Allow
    );
    assert_eq!(
        disabled(&["edit", "write", "apply_patch"], &effective),
        BTreeSet::new()
    );
}

#[test]
#[ignore = "porting: agent.subagent-permissions not implemented"]
fn subagent_self_permissions_are_preserved() {
    let executor = test_agent(
        "executor",
        AgentMode::Subagent,
        json!({
            "*": "deny",
            "read": "allow",
            "bash": "allow",
            "task": { "*": "deny", "worker": "allow" },
            "edit": "allow"
        }),
    );

    let effective = merge(
        &executor.permission,
        &derive_subagent_session_permission(&Ruleset::new(), &executor),
    );

    assert_eq!(
        evaluate("read", "README.md", &effective).action,
        Action::Allow
    );
    assert_eq!(
        evaluate("bash", "git status", &effective).action,
        Action::Allow
    );
    assert_eq!(evaluate("task", "worker", &effective).action, Action::Allow);
    assert_eq!(evaluate("task", "other", &effective).action, Action::Deny);
    assert_eq!(
        disabled(&["edit", "write", "apply_patch"], &effective),
        BTreeSet::new()
    );
}

#[test]
#[ignore = "porting: agent.subagent-permissions not implemented"]
fn subagent_inherits_parent_session_deny_rules_as_hard_runtime_ceilings() {
    let executor = test_agent("executor", AgentMode::Subagent, json!({ "bash": "allow" }));
    let effective = merge(
        &executor.permission,
        &derive_subagent_session_permission(&from_config(&json!({ "bash": "deny" })), &executor),
    );

    assert_eq!(
        evaluate("bash", "git status", &effective).action,
        Action::Deny
    );
}
