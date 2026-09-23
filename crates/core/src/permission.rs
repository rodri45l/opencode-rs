//! Permission evaluation.
//!
//! Ports the observable behaviour of `packages/core/src/permission.ts`:
//! configured rules are evaluated against an action/resource pair, an explicit
//! match decides `allow`/`deny`, `deny` takes precedence over `allow`, `*`
//! matches any resource, and no match asks.

use crate::{CoreError, CoreResult};

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
    pub fn evaluate(_rules: &[Rule], _input: &AssertInput) -> CoreResult<Effect> {
        Err(CoreError::NotImplemented(
            "permission::PermissionV2::evaluate",
        ))
    }

    /// Evaluate the configured rules against an assertion.
    pub fn ask(&self, _input: &AssertInput) -> CoreResult<Effect> {
        let _ = &self.rules;
        Err(CoreError::NotImplemented("permission::PermissionV2::ask"))
    }
}
