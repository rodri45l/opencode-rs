//! Typed live/durable events.
//!
//! Ports the definition and publication shape of `packages/core/src/event.ts`
//! (`EventV2`): a definition carries an optional durable `(version, aggregate)`
//! declaration, publishing returns an envelope tagged with the current location
//! and definition version, and [`EventService::latest`] selects the newest
//! durable definition per type regardless of declaration order.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use crate::CoreResult;

/// Durable declaration for an event definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableDef {
    /// Schema version.
    pub version: i64,
    /// Payload field carrying the aggregate id.
    pub aggregate: String,
}

/// A registered event definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDefinition {
    /// Event type name.
    pub event_type: String,
    /// Durable declaration, when the event persists.
    pub durable: Option<DurableDef>,
}

impl EventDefinition {
    /// Define an event type.
    pub fn new(event_type: impl Into<String>, durable: Option<DurableDef>) -> Self {
        Self {
            event_type: event_type.into(),
            durable,
        }
    }
}

/// Durable metadata attached to a published event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDurable {
    /// Schema version.
    pub version: i64,
    /// Per-aggregate sequence number.
    pub seq: i64,
}

/// Where a published event originated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLocation {
    /// Active directory.
    pub directory: String,
    /// Active workspace, when known.
    pub workspace_id: Option<String>,
}

/// A published event.
#[derive(Debug, Clone, PartialEq)]
pub struct EventEnvelopeV2 {
    /// Event id.
    pub id: String,
    /// Event type name.
    pub event_type: String,
    /// Durable metadata, when persisted.
    pub durable: Option<EventDurable>,
    /// Originating location, when known.
    pub location: Option<EventLocation>,
    /// Payload.
    pub data: Value,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Publishes defined events.
#[derive(Debug, Default)]
pub struct EventService {
    location: Option<EventLocation>,
}

impl EventService {
    /// Create a service, optionally bound to a location.
    pub fn new(location: Option<EventLocation>) -> Self {
        Self { location }
    }

    /// The bound location, if any.
    pub fn location(&self) -> Option<&EventLocation> {
        self.location.as_ref()
    }

    /// Publish `data` for `definition`.
    pub fn publish(
        &self,
        definition: &EventDefinition,
        data: Value,
    ) -> CoreResult<EventEnvelopeV2> {
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(EventEnvelopeV2 {
            id: format!("evt_{sequence:012}"),
            event_type: definition.event_type.clone(),
            durable: definition.durable.as_ref().map(|durable| EventDurable {
                version: durable.version,
                seq: 0,
            }),
            location: self.location.clone(),
            data,
        })
    }

    /// Select the latest durable definition per event type.
    pub fn latest(
        definitions: &[EventDefinition],
    ) -> CoreResult<BTreeMap<String, EventDefinition>> {
        let mut latest: BTreeMap<String, EventDefinition> = BTreeMap::new();
        for definition in definitions {
            match latest.get(&definition.event_type) {
                Some(existing) => {
                    let existing_version = existing
                        .durable
                        .as_ref()
                        .map(|d| d.version)
                        .unwrap_or(i64::MIN);
                    let candidate_version = definition
                        .durable
                        .as_ref()
                        .map(|d| d.version)
                        .unwrap_or(i64::MIN);
                    if candidate_version > existing_version {
                        latest.insert(definition.event_type.clone(), definition.clone());
                    }
                }
                None => {
                    latest.insert(definition.event_type.clone(), definition.clone());
                }
            }
        }
        Ok(latest)
    }
}
