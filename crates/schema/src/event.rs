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
