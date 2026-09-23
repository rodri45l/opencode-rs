//! The event envelope.
//!
//! Every server-sent event shares one envelope shape:
//!
//! ```json
//! { "id": "evt_...", "type": "session.next.step.ended", "data": { ... },
//!   "durable": { "aggregateID": "...", "seq": 1, "version": 1 },
//!   "location": { "directory": "..." } }
//! ```
//!
//! `data` is left as raw JSON for now; typed payloads are introduced per phase
//! as their behaviour is pinned by cassettes and conformance tests.

use crate::event_type::EventType;
use crate::ids::{EventId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Error returned by the event definition registries.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EventRegistryError {
    /// A module used by a ported test has not been implemented yet.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    /// Two distinct definitions claim the same type at the same version.
    #[error("duplicate latest event definition for {0}")]
    DuplicateLatest(&'static str),
    /// Two durable definitions claim the same type and version.
    #[error("duplicate durable event definition for {0}")]
    DuplicateDurable(String),
}

/// Durable metadata attached to an event definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurableSpec {
    /// Schema version of the durable payload.
    pub version: i64,
    /// Aggregate the durable event belongs to (e.g. `sessionID`).
    pub aggregate: &'static str,
}

/// A declared public event, mirroring the reference `Event.Definition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Definition {
    /// The wire event name.
    pub event_type: &'static str,
    /// Durable metadata, present only when the event is persisted.
    pub durable: Option<DurableSpec>,
}

/// Declare an event definition.
pub const fn define(event_type: &'static str, durable: Option<DurableSpec>) -> Definition {
    Definition {
        event_type,
        durable,
    }
}

/// Freeze a list of definitions into an inventory (declaration-order preserved).
pub fn inventory<'a>(definitions: &[&'a Definition]) -> Vec<&'a Definition> {
    definitions.to_vec()
}

/// The durable index key for a type/version pair.
pub fn versioned_type(event_type: &str, version: i64) -> String {
    format!("{event_type}.{version}")
}

/// Index definitions by type, selecting the highest durable version.
///
/// Mirrors `Event.latest`: declaration order must not affect the result, and
/// two distinct definitions for the same type at the same version are an error.
pub fn latest<'a>(
    definitions: &[&'a Definition],
) -> Result<BTreeMap<&'a str, &'a Definition>, EventRegistryError> {
    let mut result: BTreeMap<&'a str, &'a Definition> = BTreeMap::new();
    for definition in definitions {
        let definition = *definition;
        match result.get(definition.event_type) {
            None => {
                result.insert(definition.event_type, definition);
            }
            Some(existing) => match (definition.durable, existing.durable) {
                (Some(new_durable), Some(old_durable))
                    if new_durable.version != old_durable.version =>
                {
                    if new_durable.version > old_durable.version {
                        result.insert(definition.event_type, definition);
                    }
                }
                _ => {
                    if !std::ptr::eq(definition, *existing) {
                        return Err(EventRegistryError::DuplicateLatest(definition.event_type));
                    }
                }
            },
        }
    }
    Ok(result)
}

/// Index durable definitions by `type.version`.
pub fn durable<'a>(
    definitions: &[&'a Definition],
) -> Result<BTreeMap<String, &'a Definition>, EventRegistryError> {
    let mut result: BTreeMap<String, &'a Definition> = BTreeMap::new();
    for definition in definitions {
        let definition = *definition;
        if let Some(durable) = definition.durable {
            let key = versioned_type(definition.event_type, durable.version);
            if result.contains_key(&key) {
                return Err(EventRegistryError::DuplicateDurable(key));
            }
            result.insert(key, definition);
        }
    }
    Ok(result)
}

/// Durable-event cursor metadata. Present only when an event is persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableRef {
    #[serde(rename = "aggregateID")]
    pub aggregate_id: String,
    pub seq: i64,
    pub version: i64,
}

/// Where an event originated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationRef {
    pub directory: String,
    #[serde(
        rename = "workspaceID",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub workspace_id: Option<WorkspaceId>,
}

/// A single public event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub durable: Option<DurableRef>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub location: Option<LocationRef>,
    pub data: serde_json::Value,
}

impl EventEnvelope {
    /// Build an event with a freshly generated id.
    pub fn new(event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            id: EventId::generate(),
            metadata: None,
            event_type,
            durable: None,
            location: None,
            data,
        }
    }

    /// Attach location metadata.
    pub fn with_location(mut self, location: LocationRef) -> Self {
        self.location = Some(location);
        self
    }

    /// Mark the event durable at the given cursor.
    pub fn with_durable(mut self, durable: DurableRef) -> Self {
        self.durable = Some(durable);
        self
    }

    /// The wire event name.
    pub fn name(&self) -> &'static str {
        self.event_type.as_str()
    }
}
