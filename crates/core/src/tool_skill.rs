//! Skill tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/skill.ts`: the tool
//! is registered as `skill`, asserts permission action `skill` with the selected
//! name as both resource and save scope, loads the model-facing content
//! containing the base directory and reference files, reports
//! `Unable to load skill <name>` for an unknown or denied skill, and honors the
//! configured description. The `SkillV2`/`Permission`/`ToolRegistry` wiring and
//! the `SKILL.md` filesystem load are dropped; the pure projection remains.

use std::path::Path;

use crate::CoreResult;

/// A resolved skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    /// The skill name.
    pub name: String,
    /// The skill description.
    pub description: String,
    /// The absolute path to the skill location (its `SKILL.md`).
    pub location: String,
    /// The skill guidance content.
    pub content: String,
}

/// The skill tool.
#[derive(Debug, Default)]
pub struct SkillTool;

impl SkillTool {
    /// The tool name.
    pub const NAME: &'static str = "skill";

    /// The permission action.
    pub const ACTION: &'static str = "skill";

    /// The configured tool description.
    pub const DESCRIPTION: &'static str =
        "Load a specialized skill that provides domain-specific instructions and workflows";

    /// The permission resources for a selected skill name.
    pub fn permission_resources(name: &str) -> CoreResult<Vec<String>> {
        Ok(vec![name.to_string()])
    }

    /// The failure output for an unknown or denied skill.
    pub fn failure_message(name: &str) -> CoreResult<String> {
        Ok(format!("Unable to load skill {name}"))
    }

    /// Render the model-facing skill content.
    pub fn to_model_output(skill: &SkillInfo, reference_files: &[String]) -> CoreResult<String> {
        let base = Self::base_directory(&skill.location)?;
        let mut sections = vec![format!("Base directory for this skill: {base}")];
        sections.push(skill.content.clone());
        if !reference_files.is_empty() {
            let mut references = vec!["Reference files:".to_string()];
            references.extend(reference_files.iter().cloned());
            sections.push(references.join("\n"));
        }
        Ok(sections.join("\n\n"))
    }

    /// The base directory advertised to the model: the parent of the location.
    pub fn base_directory(location: &str) -> CoreResult<String> {
        Ok(parent_directory(location).unwrap_or_default())
    }
}

/// The directory containing a skill location.
pub fn parent_directory(location: &str) -> Option<String> {
    Path::new(location)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
}
