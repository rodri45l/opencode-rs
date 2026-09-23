//! Command definitions and transforms.
//!
//! Ports the observable behaviour of `packages/core/src/command.ts`
//! (`CommandV2`): commands are built from replayable transforms that merge into
//! the existing definition, with later updates overriding earlier ones.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::model::ModelRef;
use crate::CoreResult;

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
type CommandTransform = Box<dyn Fn(&mut CommandEditor)>;

#[derive(Default)]
pub struct CommandRegistry {
    transforms: RefCell<Vec<CommandTransform>>,
    entries: RefCell<BTreeMap<String, CommandDraft>>,
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry").finish_non_exhaustive()
    }
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> CoreResult<Self> {
        Ok(Self::default())
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, transform: F) -> CoreResult<()>
    where
        F: Fn(&mut CommandEditor) + 'static,
    {
        self.transforms.borrow_mut().push(Box::new(transform));
        self.rebuild()
    }

    fn rebuild(&self) -> CoreResult<()> {
        let mut editor = CommandEditor::new();
        for transform in self.transforms.borrow().iter() {
            transform(&mut editor);
        }
        *self.entries.borrow_mut() = editor.entries;
        Ok(())
    }

    /// Look up a command by name.
    pub fn get(&self, name: &str) -> CoreResult<Option<CommandInfo>> {
        Ok(self.entries.borrow().get(name).map(materialize))
    }

    /// List all commands.
    pub fn list(&self) -> CoreResult<Vec<CommandInfo>> {
        Ok(self.entries.borrow().values().map(materialize).collect())
    }
}

fn materialize(draft: &CommandDraft) -> CommandInfo {
    CommandInfo {
        name: draft.name.clone(),
        template: draft.template.clone().unwrap_or_default(),
        description: draft.description.clone(),
        model: draft.model.clone(),
    }
}
