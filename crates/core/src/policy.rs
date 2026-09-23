//! Policy evaluation.
//!
//! Ports the observable behaviour of `packages/core/src/policy.ts`: loaded
//! statements are evaluated in written order, the last match wins, `*` is a
//! wildcard within an action or resource segment, and no match yields the
//! caller's fallback.

use std::cell::RefCell;

use crate::CoreResult;

/// The effect of a policy statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyEffect {
    /// Explicitly allowed.
    Allow,
    /// Explicitly denied.
    Deny,
    /// Prompt the user.
    Ask,
}

impl PolicyEffect {
    /// The wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
        }
    }
}

/// One policy statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInfo {
    /// Effect when the statement matches.
    pub effect: PolicyEffect,
    /// Action pattern, e.g. `provider.*`.
    pub action: String,
    /// Resource pattern, e.g. `company-*`.
    pub resource: String,
}

impl PolicyInfo {
    /// Construct a statement.
    pub fn new(
        effect: PolicyEffect,
        action: impl Into<String>,
        resource: impl Into<String>,
    ) -> Self {
        Self {
            effect,
            action: action.into(),
            resource: resource.into(),
        }
    }
}

/// Evaluates loaded policy statements.
#[derive(Debug, Default)]
pub struct PolicyService {
    statements: RefCell<Vec<PolicyInfo>>,
}

impl PolicyService {
    /// Create an empty policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the loaded statements.
    pub fn load(&self, statements: Vec<PolicyInfo>) -> CoreResult<()> {
        *self.statements.borrow_mut() = statements;
        Ok(())
    }

    /// Evaluate `action`/`resource`, returning `fallback` when nothing matches.
    pub fn evaluate(
        &self,
        action: &str,
        resource: &str,
        fallback: PolicyEffect,
    ) -> CoreResult<PolicyEffect> {
        let mut result = fallback;
        for statement in self.statements.borrow().iter() {
            if matches_pattern(&statement.action, action)
                && matches_pattern(&statement.resource, resource)
            {
                result = statement.effect;
            }
        }
        Ok(result)
    }

    /// Whether any statements are loaded.
    pub fn has_statements(&self) -> bool {
        !self.statements.borrow().is_empty()
    }
}

fn matches_pattern(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    glob_match(pattern, value)
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
    match pattern[0] {
        '*' => {
            for index in 0..=value.len() {
                if glob(&pattern[1..], &value[index..]) {
                    return true;
                }
            }
            false
        }
        '?' => !value.is_empty() && glob(&pattern[1..], &value[1..]),
        other => !value.is_empty() && other == value[0] && glob(&pattern[1..], &value[1..]),
    }
}
