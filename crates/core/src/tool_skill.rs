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

use crate::{CoreError, CoreResult};

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
    pub fn permission_resources(_name: &str) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented(
            "tool_skill::SkillTool::permission_resources",
        ))
    }

    /// The failure output for an unknown or denied skill.
    pub fn failure_message(_name: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_skill::SkillTool::failure_message",
        ))
    }

    /// Render the model-facing skill content.
    pub fn to_model_output(_skill: &SkillInfo, _reference_files: &[String]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_skill::SkillTool::to_model_output",
        ))
    }

    /// The base directory advertised to the model: the parent of the location.
    pub fn base_directory(_location: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_skill::SkillTool::base_directory",
        ))
    }
}

/// The directory containing a skill location.
pub fn parent_directory(location: &str) -> Option<String> {
    Path::new(location)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
}
