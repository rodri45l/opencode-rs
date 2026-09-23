//! Agent definitions and replayable transforms.
//!
//! Ports the observable behaviour of `packages/core/src/agent.ts` (`AgentV2`):
//! a registry of agents built from replayable transforms, with direct
//! create/update/remove and a `reload` that re-applies every transform.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::CoreResult;

/// Identifier for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(String);

impl AgentId {
    /// Construct an agent id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether an agent can be selected directly or only by another agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    /// Selectable as the top-level agent.
    Primary,
    /// Only invocable as a subagent.
    Subagent,
}

/// A single permission rule attached to an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRule {
    /// The action the rule governs, e.g. `bash`.
    pub action: String,
    /// The effect, e.g. `allow` / `ask` / `deny`.
    pub effect: String,
}

/// A materialized agent definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInfo {
    /// Agent id.
    pub id: AgentId,
    /// Human description.
    pub description: String,
    /// Primary or subagent.
    pub mode: AgentMode,
    /// Whether the agent is hidden from the picker.
    pub hidden: bool,
    /// Permission rules.
    pub permissions: Vec<PermissionRule>,
}

impl AgentInfo {
    /// The runtime-default agent for an id.
    pub fn empty(id: AgentId) -> CoreResult<Self> {
        Ok(Self {
            id,
            description: String::new(),
            mode: AgentMode::Subagent,
            hidden: false,
            permissions: Vec::new(),
        })
    }
}

/// Mutable draft handed to a transform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDraft {
    /// Agent id.
    pub id: AgentId,
    /// Description, when set.
    pub description: Option<String>,
    /// Mode, when set.
    pub mode: Option<AgentMode>,
    /// Hidden flag, when set.
    pub hidden: Option<bool>,
    /// Permission rules.
    pub permissions: Vec<PermissionRule>,
}

impl AgentDraft {
    fn for_id(id: AgentId) -> Self {
        Self {
            id,
            description: None,
            mode: None,
            hidden: None,
            permissions: Vec::new(),
        }
    }
}

/// Editor surface exposed to a transform.
#[derive(Debug, Default)]
pub struct AgentEditor {
    entries: BTreeMap<String, AgentDraft>,
    removed: BTreeSet<String>,
}

impl AgentEditor {
    /// Create an empty editor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update (creating if needed) the agent with `id`.
    pub fn update<F>(&mut self, id: AgentId, update: F)
    where
        F: FnOnce(&mut AgentDraft),
    {
        let key = id.as_str().to_string();
        let draft = self
            .entries
            .entry(key.clone())
            .or_insert_with(|| AgentDraft::for_id(id));
        update(draft);
        self.removed.remove(&key);
    }

    /// Remove the agent with `id`.
    pub fn remove(&mut self, id: AgentId) {
        let key = id.as_str().to_string();
        self.entries.remove(&key);
        self.removed.insert(key);
    }
}

/// Registry of agents built from replayable transforms.
type AgentTransform = Box<dyn Fn(&mut AgentEditor)>;

#[derive(Default)]
pub struct AgentRegistry {
    transforms: std::cell::RefCell<Vec<AgentTransform>>,
    entries: std::cell::RefCell<BTreeMap<String, AgentDraft>>,
}

impl std::fmt::Debug for AgentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentRegistry").finish_non_exhaustive()
    }
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> CoreResult<Self> {
        Ok(Self::default())
    }

    /// All materialized agents.
    pub fn all(&self) -> CoreResult<Vec<AgentInfo>> {
        Ok(self.entries.borrow().values().map(materialize).collect())
    }

    /// Look up one agent.
    pub fn get(&self, id: &AgentId) -> CoreResult<Option<AgentInfo>> {
        Ok(self.entries.borrow().get(id.as_str()).map(materialize))
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, transform: F) -> CoreResult<()>
    where
        F: Fn(&mut AgentEditor) + 'static,
    {
        self.transforms.borrow_mut().push(Box::new(transform));
        self.rebuild()
    }

    /// Re-apply every registered transform.
    pub fn reload(&self) -> CoreResult<()> {
        self.rebuild()
    }

    fn rebuild(&self) -> CoreResult<()> {
        let mut editor = AgentEditor::new();
        for transform in self.transforms.borrow().iter() {
            transform(&mut editor);
        }
        *self.entries.borrow_mut() = editor.entries;
        Ok(())
    }
}

fn materialize(draft: &AgentDraft) -> AgentInfo {
    AgentInfo {
        id: draft.id.clone(),
        description: draft.description.clone().unwrap_or_default(),
        mode: draft.mode.unwrap_or(AgentMode::Subagent),
        hidden: draft.hidden.unwrap_or(false),
        permissions: draft.permissions.clone(),
    }
}
