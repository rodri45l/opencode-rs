//! Isolated V1 schema subtree.
//!
//! Mirrors `packages/schema/src/v1/*`. Current code must reach this subtree only
//! through the compatibility entrypoints (`legacy_event`, `permission_v1`,
//! `question_v1`, `session_v1`); see `tests/v1_isolation.rs`.

use crate::event::{define, Definition, DurableSpec};

/// `packages/schema/src/v1/legacy-event.ts`.
pub mod legacy_event {
    use super::*;

    /// `command.executed`.
    pub const COMMAND_EXECUTED: Definition = define("command.executed", None);

    /// All legacy-event definitions.
    pub static DEFINITIONS: &[Definition] = &[COMMAND_EXECUTED];

    /// Marker preserving the reference namespace identity.
    pub struct LegacyEvent;
}

/// `packages/schema/src/v1/permission.ts` event surface.
pub mod permission {
    use super::*;

    /// `permission.asked`.
    pub const ASKED: Definition = define("permission.asked", None);
    /// `permission.replied`.
    pub const REPLIED: Definition = define("permission.replied", None);

    /// V1 permission events.
    pub mod event {
        use crate::event::Definition;

        pub const ASKED: Definition = super::ASKED;
        pub const REPLIED: Definition = super::REPLIED;
        pub static DEFINITIONS: &[Definition] = &[ASKED, REPLIED];
    }

    /// Marker preserving the reference namespace identity.
    pub struct PermissionV1;
}

/// `packages/schema/src/v1/question.ts` event surface.
pub mod question {
    use super::*;

    /// `question.asked`.
    pub const ASKED: Definition = define("question.asked", None);
    /// `question.replied`.
    pub const REPLIED: Definition = define("question.replied", None);
    /// `question.rejected`.
    pub const REJECTED: Definition = define("question.rejected", None);

    /// V1 question events.
    pub mod event {
        use crate::event::Definition;

        pub const ASKED: Definition = super::ASKED;
        pub const REPLIED: Definition = super::REPLIED;
        pub const REJECTED: Definition = super::REJECTED;
        pub static DEFINITIONS: &[Definition] = &[ASKED, REPLIED, REJECTED];
    }

    /// Marker preserving the reference namespace identity.
    pub struct QuestionV1;
}

/// `packages/schema/src/v1/session.ts` event surface.
pub mod session {
    use super::*;

    const DURABLE: DurableSpec = DurableSpec {
        version: 1,
        aggregate: "sessionID",
    };

    /// `session.created`.
    pub const CREATED: Definition = define("session.created", Some(DURABLE));
    /// `session.updated`.
    pub const UPDATED: Definition = define("session.updated", Some(DURABLE));
    /// `session.deleted`.
    pub const DELETED: Definition = define("session.deleted", Some(DURABLE));
    /// `message.updated`.
    pub const MESSAGE_UPDATED: Definition = define("message.updated", Some(DURABLE));
    /// `message.removed`.
    pub const MESSAGE_REMOVED: Definition = define("message.removed", Some(DURABLE));
    /// `message.part.updated`.
    pub const PART_UPDATED: Definition = define("message.part.updated", Some(DURABLE));
    /// `message.part.removed`.
    pub const PART_REMOVED: Definition = define("message.part.removed", Some(DURABLE));
    /// `message.part.delta`.
    pub const PART_DELTA: Definition = define("message.part.delta", None);
    /// `session.diff`.
    pub const DIFF: Definition = define("session.diff", None);
    /// `session.error`.
    pub const ERROR: Definition = define("session.error", None);

    /// V1 session events.
    pub mod event {
        use crate::event::Definition;

        pub const CREATED: Definition = super::CREATED;
        pub const UPDATED: Definition = super::UPDATED;
        pub const DELETED: Definition = super::DELETED;
        pub const MESSAGE_UPDATED: Definition = super::MESSAGE_UPDATED;
        pub const MESSAGE_REMOVED: Definition = super::MESSAGE_REMOVED;
        pub const PART_UPDATED: Definition = super::PART_UPDATED;
        pub const PART_REMOVED: Definition = super::PART_REMOVED;
        pub const PART_DELTA: Definition = super::PART_DELTA;
        pub const DIFF: Definition = super::DIFF;
        pub const ERROR: Definition = super::ERROR;

        pub static DEFINITIONS: &[Definition] = &[
            CREATED,
            UPDATED,
            DELETED,
            MESSAGE_UPDATED,
            MESSAGE_REMOVED,
            PART_UPDATED,
            PART_REMOVED,
            PART_DELTA,
            DIFF,
            ERROR,
        ];
    }

    /// Marker preserving the reference namespace identity.
    pub struct SessionV1;
}
