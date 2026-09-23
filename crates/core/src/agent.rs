//! Agent definitions and replayable transforms.
//!
//! Ports the observable behaviour of `packages/core/src/agent.ts` (`AgentV2`):
//! a registry of agents built from replayable transforms, with direct
//! create/update/remove and a `reload` that re-applies every transform.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{CoreError, CoreResult};

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
    pub fn empty(_id: AgentId) -> CoreResult<Self> {
        Err(CoreError::NotImplemented("agent::AgentInfo::empty"))
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
#[derive(Debug, Default)]
pub struct AgentRegistry;

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> CoreResult<Self> {
        Err(CoreError::NotImplemented("agent::AgentRegistry::new"))
    }

    /// All materialized agents.
    pub fn all(&self) -> CoreResult<Vec<AgentInfo>> {
        Err(CoreError::NotImplemented("agent::AgentRegistry::all"))
    }

    /// Look up one agent.
    pub fn get(&self, _id: &AgentId) -> CoreResult<Option<AgentInfo>> {
        Err(CoreError::NotImplemented("agent::AgentRegistry::get"))
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, _transform: F) -> CoreResult<()>
    where
        F: Fn(&mut AgentEditor) + 'static,
    {
        Err(CoreError::NotImplemented("agent::AgentRegistry::transform"))
    }

    /// Re-apply every registered transform.
    pub fn reload(&self) -> CoreResult<()> {
        Err(CoreError::NotImplemented("agent::AgentRegistry::reload"))
    }
}
