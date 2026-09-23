//! Policy evaluation.
//!
//! Ports the observable behaviour of `packages/core/src/policy.ts`: loaded
//! statements are evaluated in written order, the last match wins, `*` is a
//! wildcard within an action or resource segment, and no match yields the
//! caller's fallback.

use crate::{CoreError, CoreResult};

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
pub struct PolicyService;

impl PolicyService {
    /// Create an empty policy.
    pub fn new() -> Self {
        Self
    }

    /// Replace the loaded statements.
    pub fn load(&self, _statements: Vec<PolicyInfo>) -> CoreResult<()> {
        Err(CoreError::NotImplemented("policy::PolicyService::load"))
    }

    /// Evaluate `action`/`resource`, returning `fallback` when nothing matches.
    pub fn evaluate(
        &self,
        _action: &str,
        _resource: &str,
        _fallback: PolicyEffect,
    ) -> CoreResult<PolicyEffect> {
        Err(CoreError::NotImplemented("policy::PolicyService::evaluate"))
    }
}
