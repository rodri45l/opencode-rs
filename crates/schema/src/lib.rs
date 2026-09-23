//! Wire types for the opencode server.
//!
//! Types here mirror `packages/schema` in the reference implementation. The
//! [`EventType`] surface is generated from a pinned contract fixture; see
//! `tests/fixtures/events.json` and `scripts/gen_event_enum.py`.

pub mod event;
pub mod event_type;
pub mod ids;

pub use event::{DurableRef, EventEnvelope, LocationRef};
pub use event_type::{EventType, EVENT_TYPE_COUNT};
pub use ids::{
    EventId, IdError, MessageId, PartId, PermissionId, PtyId, QuestionId, SessionId, WorkspaceId,
};
