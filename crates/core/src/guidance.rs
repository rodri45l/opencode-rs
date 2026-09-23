//! Skill guidance system-context provider (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/test/skill/guidance.test.ts`:
//! given the available skills and the selected agent's permission rules, render
//! the `<available_skills>` guidance block, apply ordered permission precedence
//! (last matching rule wins), and omit the block entirely when the agent denies
//! all skills without a later specific allow. The `SystemContext` reconciliation
//! and Effect layers are dropped; the pure render remains.

use crate::CoreResult;

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
    pub fn is_available(name: &str, permissions: &[SkillPermission]) -> CoreResult<bool> {
        Ok(evaluate(name, permissions).effect != "deny")
    }

    /// Render the guidance baseline, or an empty string when guidance is omitted.
    pub fn render(skills: &[SkillInfo], permissions: &[SkillPermission]) -> CoreResult<String> {
        let mut permitted: Vec<&SkillInfo> = skills
            .iter()
            .filter(|skill| Self::is_available(&skill.name, permissions).unwrap_or(false))
            .collect();
        if permitted.is_empty() && evaluate("*", permissions).effect == "deny" {
            return Ok(String::new());
        }
        permitted.sort_by(|left, right| left.name.cmp(&right.name));
        let advertised: Vec<&SkillInfo> = permitted
            .into_iter()
            .filter(|skill| skill.description.is_some())
            .collect();
        Ok(render(&advertised))
    }
}

struct Decision {
    effect: String,
}

fn evaluate(resource: &str, permissions: &[SkillPermission]) -> Decision {
    let mut effect = "allow".to_string();
    for permission in permissions {
        if permission.resource == resource || permission.resource == "*" {
            effect = permission.effect.clone();
        }
    }
    Decision { effect }
}

fn render(skills: &[&SkillInfo]) -> String {
    let mut lines = vec![
        "Skills provide specialized instructions and workflows for specific tasks.".to_string(),
        "Use the skill tool to load a skill when a task matches its description.".to_string(),
    ];
    if skills.is_empty() {
        lines.push("No skills are currently available.".to_string());
    } else {
        lines.push("<available_skills>".to_string());
        for skill in skills {
            lines.push("  <skill>".to_string());
            lines.push(format!("    <name>{}</name>", skill.name));
            lines.push(format!(
                "    <description>{}</description>",
                skill.description.clone().unwrap_or_default()
            ));
            lines.push("  </skill>".to_string());
        }
        lines.push("</available_skills>".to_string());
    }
    lines.join("\n")
}
