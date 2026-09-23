//! Built-in skill plugin.
//!
//! Ports the observable behaviour of `packages/core/src/plugin/skill.ts`: the
//! plugin registers the `customize-opencode` skill describing how to edit
//! opencode's own configuration.

use crate::{CoreError, CoreResult};

/// A registered skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinSkill {
    /// Skill name.
    pub name: String,
    /// Skill description.
    pub description: String,
}

/// The built-in skill provider plugin.
#[derive(Debug, Default)]
pub struct PluginSkill;

impl PluginSkill {
    /// The built-in customize-opencode skill.
    pub fn builtin() -> CoreResult<BuiltinSkill> {
        Err(CoreError::NotImplemented(
            "plugin_skill::PluginSkill::builtin",
        ))
    }
}
