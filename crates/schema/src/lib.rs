//! Wire types for the opencode server.
//!
//! Types here mirror `packages/schema` in the reference implementation. The
//! [`EventType`] surface is generated from a pinned contract fixture; see
//! `tests/fixtures/events.json` and `scripts/gen_event_enum.py`.

pub mod definitions;
pub mod event;
pub mod event_manifest;
pub mod event_type;
pub mod identifiers;
pub mod ids;
pub mod legacy_event;
pub mod permission_v1;
pub mod question_v1;
pub mod schema;
pub mod session_v1;
pub mod v1;

pub use definitions::{
    filesystem, ide_event, integration, permission, project, reference, session, session_event,
    session_todo, workspace, workspace_event,
};
pub use event::{
    Definition, DurableRef, DurableSpec, EventEnvelope, EventRegistryError, LocationRef,
};
pub use event_type::{EventType, EVENT_TYPE_COUNT};
pub use ids::{
    EventId, IdError, MessageId, PartId, PermissionId, PtyId, QuestionId, SessionId, WorkspaceId,
};
