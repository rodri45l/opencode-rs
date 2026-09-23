//! Skill guidance system-context provider (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/test/skill/guidance.test.ts`:
//! given the available skills and the selected agent's permission rules, render
//! the `<available_skills>` guidance block, apply ordered permission precedence
//! (last matching rule wins), and omit the block entirely when the agent denies
//! all skills without a later specific allow. The `SystemContext` reconciliation
//! and Effect layers are dropped; the pure render remains.

use crate::{CoreError, CoreResult};

/// A skill advertised to the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    /// Skill name.
    pub name: String,
    /// Optional description; undescribed skills are not advertised.
    pub description: Option<String>,
    /// Skill content.
    pub content: String,
    /// Absolute location of the skill document.
    pub location: String,
}

/// A permission rule for the `skill` action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillPermission {
    /// Resource the rule matches (`*` is the wildcard).
    pub resource: String,
    /// `"allow"` or `"deny"`.
    pub effect: String,
}

/// Skill guidance rendering.
#[derive(Debug, Default)]
pub struct SkillGuidance;

impl SkillGuidance {
    /// Whether a skill is available to the agent after ordered permission rules.
    pub fn is_available(_name: &str, _permissions: &[SkillPermission]) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "guidance::SkillGuidance::is_available",
        ))
    }

    /// Render the guidance baseline, or an empty string when guidance is omitted.
    pub fn render(_skills: &[SkillInfo], _permissions: &[SkillPermission]) -> CoreResult<String> {
        Err(CoreError::NotImplemented("guidance::SkillGuidance::render"))
    }
}
