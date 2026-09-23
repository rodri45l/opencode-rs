//! Command definitions and transforms.
//!
//! Ports the observable behaviour of `packages/core/src/command.ts`
//! (`CommandV2`): commands are built from replayable transforms that merge into
//! the existing definition, with later updates overriding earlier ones.

use std::collections::BTreeMap;

use crate::model::ModelRef;
use crate::{CoreError, CoreResult};

/// A materialized command definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    /// Command name.
    pub name: String,
    /// Prompt template.
    pub template: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional model override.
    pub model: Option<ModelRef>,
}

/// Mutable draft handed to a command transform.
#[derive(Debug, Clone)]
pub struct CommandDraft {
    /// Command name.
    pub name: String,
    /// Prompt template.
    pub template: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Optional model override.
    pub model: Option<ModelRef>,
}

impl CommandDraft {
    fn for_name(name: String) -> Self {
        Self {
            name,
            template: None,
            description: None,
            model: None,
        }
    }
}

/// Editor surface exposed to a command transform.
#[derive(Debug, Default)]
pub struct CommandEditor {
    entries: BTreeMap<String, CommandDraft>,
}

impl CommandEditor {
    /// Create an empty editor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update (creating if needed) the command named `name`.
    pub fn update<F>(&mut self, name: &str, update: F)
    where
        F: FnOnce(&mut CommandDraft),
    {
        let draft = self
            .entries
            .entry(name.to_string())
            .or_insert_with(|| CommandDraft::for_name(name.to_string()));
        update(draft);
    }
}

/// Registry of commands built from transforms.
#[derive(Debug, Default)]
pub struct CommandRegistry;

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> CoreResult<Self> {
        Err(CoreError::NotImplemented("command::CommandRegistry::new"))
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, _transform: F) -> CoreResult<()>
    where
        F: Fn(&mut CommandEditor) + 'static,
    {
        Err(CoreError::NotImplemented(
            "command::CommandRegistry::transform",
        ))
    }

    /// Look up a command by name.
    pub fn get(&self, _name: &str) -> CoreResult<Option<CommandInfo>> {
        Err(CoreError::NotImplemented("command::CommandRegistry::get"))
    }

    /// List all commands.
    pub fn list(&self) -> CoreResult<Vec<CommandInfo>> {
        Err(CoreError::NotImplemented("command::CommandRegistry::list"))
    }
}
