//! The public event manifest.
//!
//! Mirrors `packages/schema/src/event-manifest.ts`. The registry composes every
//! module's definitions, selects the latest per type, and indexes durable
//! definitions by `type.version`. Not implemented yet; the ported contract tests
//! in `tests/event_manifest_surface.rs` are ignored until it lands.

use crate::event::{Definition, EventRegistryError};
use std::collections::BTreeMap;

/// The `ServerDefinitions` inventory (foundation + feature + todo).
pub fn server_definitions() -> Result<Vec<&'static Definition>, EventRegistryError> {
    Err(EventRegistryError::NotImplemented("event manifest"))
}

/// The `Definitions` inventory.
pub fn definitions() -> Result<Vec<&'static Definition>, EventRegistryError> {
    Err(EventRegistryError::NotImplemented("event manifest"))
}

/// The `Latest` index (highest durable version per type).
pub fn latest() -> Result<BTreeMap<&'static str, &'static Definition>, EventRegistryError> {
    Err(EventRegistryError::NotImplemented("event manifest"))
}

/// The `Durable` index, keyed by `type.version`.
pub fn durable() -> Result<BTreeMap<String, &'static Definition>, EventRegistryError> {
    Err(EventRegistryError::NotImplemented("event manifest"))
}
