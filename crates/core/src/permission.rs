//! Permission evaluation.
//!
//! Ports the observable behaviour of `packages/core/src/permission.ts`:
//! configured rules are evaluated against an action/resource pair, an explicit
//! match decides `allow`/`deny`, `deny` takes precedence over `allow`, `*`
//! matches any resource, and no match asks.

use crate::CoreResult;

/// The evaluated effect for a permission assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// The action is allowed.
    Allow,
    /// The action is denied.
    Deny,
    /// The action requires a prompt.
    Ask,
}

/// A permission rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Action name (`read`, `bash`, `*`, ...).
    pub action: String,
    /// Resource pattern (`*` matches anything).
    pub resource: String,
    /// The effect to apply.
    pub effect: Effect,
}

/// A permission assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertInput {
    /// Action name.
    pub action: String,
    /// Resources being accessed.
    pub resources: Vec<String>,
}

/// The permission service.
#[derive(Debug, Default)]
pub struct PermissionV2 {
    rules: Vec<Rule>,
}

impl PermissionV2 {
    /// Create a service with the given rules.
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    /// Evaluate rules against an assertion.
    pub fn evaluate(rules: &[Rule], input: &AssertInput) -> CoreResult<Effect> {
        let mut result: Option<Effect> = None;
        for resource in &input.resources {
            let mut effect: Option<Effect> = None;
            for rule in rules {
                if action_matches(&rule.action, &input.action)
                    && resource_matches(&rule.resource, resource)
                {
                    effect = Some(rule.effect);
                }
            }
            match effect {
                Some(Effect::Deny) => return Ok(Effect::Deny),
                Some(Effect::Allow) => result = Some(Effect::Allow),
                Some(Effect::Ask) => {
                    result.get_or_insert(Effect::Ask);
                }
                None => {
                    result.get_or_insert(Effect::Ask);
                }
            }
        }
        Ok(result.unwrap_or(Effect::Ask))
    }

    /// Evaluate the configured rules against an assertion.
    pub fn ask(&self, input: &AssertInput) -> CoreResult<Effect> {
        Self::evaluate(&self.rules, input)
    }
}

fn action_matches(pattern: &str, action: &str) -> bool {
    pattern == "*" || pattern == action
}

fn resource_matches(pattern: &str, resource: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    glob_match(pattern, resource)
}

fn glob_match(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();
    glob(&pattern, &value)
}

fn glob(pattern: &[char], value: &[char]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }
    if pattern[0] == '*' {
        for index in 0..=value.len() {
            if glob(&pattern[1..], &value[index..]) {
                return true;
            }
        }
        return false;
    }
    if pattern[0] == '?' {
        return !value.is_empty() && glob(&pattern[1..], &value[1..]);
    }
    !value.is_empty() && pattern[0] == value[0] && glob(&pattern[1..], &value[1..])
}
