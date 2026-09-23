//! Plugin registry (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin.ts`: plugins are
//! keyed by id and added, replaced, and removed by id; adding an id that already
//! exists replaces the previously applied contributions, and removing an id
//! reverts them. The Effect `PluginV2` service, `wait` semantics, activation
//! defects, and agent-transform wiring are dropped; the pure registry remains.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

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
    pub fn add(&mut self, _id: &str, _description: &str) -> CoreResult<()> {
        let _ = &mut self.applied;
        Err(CoreError::NotImplemented("plugin::PluginRegistry::add"))
    }

    /// Remove a plugin, reverting its contributions.
    pub fn remove(&mut self, _id: &str) -> CoreResult<()> {
        let _ = &mut self.applied;
        Err(CoreError::NotImplemented("plugin::PluginRegistry::remove"))
    }

    /// The description currently applied by `id`, if any.
    pub fn applied_description(&self, _id: &str) -> CoreResult<Option<String>> {
        let _ = &self.applied;
        Err(CoreError::NotImplemented(
            "plugin::PluginRegistry::applied_description",
        ))
    }
}
