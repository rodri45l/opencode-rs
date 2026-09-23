//! Port of packages/core/test/permission.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `ask` returns the evaluated effect without queueing, an
//! explicit rule decides `allow`/`deny`, `*` matches any resource, deny
//! precedence holds, and no matching rule asks. Re-derived: the Database/
//! Event/Agent/SessionStore wiring and the async prompt lifecycle are replaced by
//! direct rule evaluation.

use opencode_core::permission::{AssertInput, Effect, PermissionV2, Rule};

fn read(resource: &str) -> AssertInput {
    AssertInput {
        action: "read".into(),
        resources: vec![resource.into()],
    }
}

fn rule(action: &str, resource: &str, effect: Effect) -> Rule {
    Rule {
        action: action.into(),
        resource: resource.into(),
        effect,
    }
}

#[test]
fn returns_the_evaluated_effect() {
    let allow = PermissionV2::new(vec![rule("read", "*", Effect::Allow)]);
    assert_eq!(allow.ask(&read("src/index.ts")).unwrap(), Effect::Allow);

    let deny = PermissionV2::new(vec![rule("read", "*", Effect::Deny)]);
    assert_eq!(deny.ask(&read("src/index.ts")).unwrap(), Effect::Deny);

    let unset = PermissionV2::new(vec![]);
    assert_eq!(unset.ask(&read("src/index.ts")).unwrap(), Effect::Ask);
}

#[test]
fn lets_deny_take_precedence_over_a_broader_allow() {
    let service = PermissionV2::new(vec![
        rule("*", "*", Effect::Allow),
        rule("read", "src/secret.ts", Effect::Deny),
    ]);

    assert_eq!(service.ask(&read("src/index.ts")).unwrap(), Effect::Allow);
    assert_eq!(service.ask(&read("src/secret.ts")).unwrap(), Effect::Deny);
}

#[test]
fn matches_a_wildcard_resource_for_a_specific_action() {
    let service = PermissionV2::new(vec![rule("bash", "*", Effect::Allow)]);
    let bash = AssertInput {
        action: "bash".into(),
        resources: vec!["pwd".into()],
    };
    assert_eq!(service.ask(&bash).unwrap(), Effect::Allow);
}
