//! Typed live/durable events.
//!
//! Ports the definition and publication shape of `packages/core/src/event.ts`
//! (`EventV2`): a definition carries an optional durable `(version, aggregate)`
//! declaration, publishing returns an envelope tagged with the current location
//! and definition version, and [`EventService::latest`] selects the newest
//! durable definition per type regardless of declaration order.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

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
        _definition: &EventDefinition,
        _data: Value,
    ) -> CoreResult<EventEnvelopeV2> {
        Err(CoreError::NotImplemented("event::EventService::publish"))
    }

    /// Select the latest durable definition per event type.
    pub fn latest(
        _definitions: &[EventDefinition],
    ) -> CoreResult<BTreeMap<String, EventDefinition>> {
        Err(CoreError::NotImplemented("event::EventService::latest"))
    }
}
