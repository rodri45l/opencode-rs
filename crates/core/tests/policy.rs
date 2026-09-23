//! Port of packages/core/test/policy.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: no match returns the caller's fallback; wildcard
//! action/resource matching is written-order sensitive; and the last matching
//! statement wins.

use opencode_core::policy::{PolicyEffect, PolicyInfo, PolicyService};

#[test]
fn returns_the_callers_fallback_when_no_statement_matches() {
    let policy = PolicyService::new();

    assert_eq!(
        policy
            .evaluate("provider.use", "anthropic", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Allow
    );
    assert_eq!(
        policy
            .evaluate("provider.use", "anthropic", PolicyEffect::Deny)
            .unwrap(),
        PolicyEffect::Deny
    );
}

#[test]
fn evaluates_wildcard_provider_rules_in_written_order() {
    let policy = PolicyService::new();
    policy
        .load(vec![
            PolicyInfo::new(PolicyEffect::Deny, "provider.*", "*"),
            PolicyInfo::new(PolicyEffect::Allow, "provider.use", "anthropic"),
        ])
        .unwrap();

    assert_eq!(
        policy
            .evaluate("provider.use", "anthropic", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Allow
    );
    assert_eq!(
        policy
            .evaluate("provider.use", "openai", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Deny
    );
}

#[test]
fn matches_action_and_resource_independently() {
    let policy = PolicyService::new();
    policy
        .load(vec![PolicyInfo::new(
            PolicyEffect::Deny,
            "provider.*",
            "company-*",
        )])
        .unwrap();

    assert_eq!(
        policy
            .evaluate("provider.use", "company-stable", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Deny
    );
    assert_eq!(
        policy
            .evaluate("plugin.load", "company-stable", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Allow
    );
}

#[test]
fn uses_the_last_matching_loaded_statement() {
    let policy = PolicyService::new();
    policy
        .load(vec![
            PolicyInfo::new(PolicyEffect::Allow, "provider.use", "openai"),
            PolicyInfo::new(PolicyEffect::Deny, "provider.use", "openai"),
        ])
        .unwrap();

    assert_eq!(
        policy
            .evaluate("provider.use", "openai", PolicyEffect::Allow)
            .unwrap(),
        PolicyEffect::Deny
    );
}
