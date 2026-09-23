//! Plugin registry (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin.ts`: plugins are
//! keyed by id and added, replaced, and removed by id; adding an id that already
//! exists replaces the previously applied contributions, and removing an id
//! reverts them. The Effect `PluginV2` service, `wait` semantics, activation
//! defects, and agent-transform wiring are dropped; the pure registry remains.

use std::collections::BTreeMap;

use crate::CoreResult;

/// An in-memory plugin registry keyed by plugin id.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    applied: BTreeMap<String, String>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace a plugin, applying its description.
    pub fn add(&mut self, id: &str, description: &str) -> CoreResult<()> {
        self.applied.insert(id.to_string(), description.to_string());
        Ok(())
    }

    /// Remove a plugin, reverting its contributions.
    pub fn remove(&mut self, id: &str) -> CoreResult<()> {
        self.applied.remove(id);
        Ok(())
    }

    /// The description currently applied by `id`, if any.
    pub fn applied_description(&self, id: &str) -> CoreResult<Option<String>> {
        Ok(self.applied.get(id).cloned())
    }
}
