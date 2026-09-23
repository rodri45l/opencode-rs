//! Event definition registries for the current (v2) public surface.
//!
//! Ports the definition lists declared by the reference
//! `packages/schema/src/*.ts` modules. Only the type discriminators and durable
//! metadata are modelled here; payload schemas land with the modules that
//! consume them.

use crate::event::{define, Definition, DurableSpec};

const SESSION_V1: DurableSpec = DurableSpec {
    version: 1,
    aggregate: "sessionID",
};
const SESSION_V2: DurableSpec = DurableSpec {
    version: 2,
    aggregate: "sessionID",
};

/// `packages/schema/src/session-event.ts`.
pub mod session_event {
    use super::*;

    /// Marker preserving the reference namespace identity.
    pub struct SessionEvent;

    /// `session.next.step.*` definitions.
    pub mod step {
        use super::*;

        /// `session.next.step.ended`.
        pub const ENDED: Definition = define("session.next.step.ended", Some(SESSION_V2));
    }

    pub const AGENT_SWITCHED: Definition = define("session.next.agent.switched", Some(SESSION_V1));
    pub const MODEL_SWITCHED: Definition = define("session.next.model.switched", Some(SESSION_V1));
    pub const MOVED: Definition = define("session.next.moved", Some(SESSION_V1));
    pub const PROMPTED: Definition = define("session.next.prompted", Some(SESSION_V1));
    pub const PROMPT_ADMITTED: Definition =
        define("session.next.prompt.admitted", Some(SESSION_V1));
    pub const CONTEXT_UPDATED: Definition =
        define("session.next.context.updated", Some(SESSION_V1));
    pub const SYNTHETIC: Definition = define("session.next.synthetic", Some(SESSION_V1));
    pub const SHELL_STARTED: Definition = define("session.next.shell.started", Some(SESSION_V1));
    pub const SHELL_ENDED: Definition = define("session.next.shell.ended", Some(SESSION_V1));
    pub const STEP_STARTED: Definition = define("session.next.step.started", Some(SESSION_V1));
    pub const STEP_ENDED: Definition = define("session.next.step.ended", Some(SESSION_V2));
    pub const STEP_FAILED: Definition = define("session.next.step.failed", Some(SESSION_V2));
    pub const TEXT_STARTED: Definition = define("session.next.text.started", Some(SESSION_V1));
    pub const TEXT_DELTA: Definition = define("session.next.text.delta", None);
    pub const TEXT_ENDED: Definition = define("session.next.text.ended", Some(SESSION_V1));
    pub const REASONING_STARTED: Definition =
        define("session.next.reasoning.started", Some(SESSION_V1));
    pub const REASONING_DELTA: Definition = define("session.next.reasoning.delta", None);
    pub const REASONING_ENDED: Definition =
        define("session.next.reasoning.ended", Some(SESSION_V1));
    pub const TOOL_INPUT_STARTED: Definition =
        define("session.next.tool.input.started", Some(SESSION_V1));
    pub const TOOL_INPUT_DELTA: Definition = define("session.next.tool.input.delta", None);
    pub const TOOL_INPUT_ENDED: Definition =
        define("session.next.tool.input.ended", Some(SESSION_V1));
    pub const TOOL_CALLED: Definition = define("session.next.tool.called", Some(SESSION_V1));
    pub const TOOL_PROGRESS: Definition = define("session.next.tool.progress", Some(SESSION_V1));
    pub const TOOL_SUCCESS: Definition = define("session.next.tool.success", Some(SESSION_V1));
    pub const TOOL_FAILED: Definition = define("session.next.tool.failed", Some(SESSION_V1));
    pub const RETRIED: Definition = define("session.next.retried", Some(SESSION_V1));
    pub const COMPACTION_STARTED: Definition =
        define("session.next.compaction.started", Some(SESSION_V1));
    pub const COMPACTION_DELTA: Definition = define("session.next.compaction.delta", None);
    pub const COMPACTION_ENDED: Definition =
        define("session.next.compaction.ended", Some(SESSION_V1));
    pub const REVERT_STAGED: Definition = define("session.next.revert.staged", Some(SESSION_V1));
    pub const REVERT_CLEARED: Definition = define("session.next.revert.cleared", Some(SESSION_V1));
    pub const REVERT_COMMITTED: Definition =
        define("session.next.revert.committed", Some(SESSION_V1));

    /// All current definitions, in reference declaration order.
    pub static DEFINITIONS: &[Definition] = &[
        AGENT_SWITCHED,
        MODEL_SWITCHED,
        MOVED,
        PROMPTED,
        PROMPT_ADMITTED,
        CONTEXT_UPDATED,
        SYNTHETIC,
        SHELL_STARTED,
        SHELL_ENDED,
        STEP_STARTED,
        STEP_ENDED,
        STEP_FAILED,
        TEXT_STARTED,
        TEXT_DELTA,
        TEXT_ENDED,
        REASONING_STARTED,
        REASONING_DELTA,
        REASONING_ENDED,
        TOOL_INPUT_STARTED,
        TOOL_INPUT_DELTA,
        TOOL_INPUT_ENDED,
        TOOL_CALLED,
        TOOL_PROGRESS,
        TOOL_SUCCESS,
        TOOL_FAILED,
        RETRIED,
        COMPACTION_STARTED,
        COMPACTION_DELTA,
        COMPACTION_ENDED,
        REVERT_STAGED,
        REVERT_CLEARED,
        REVERT_COMMITTED,
    ];
}

/// `packages/schema/src/workspace-event.ts`.
pub mod workspace_event {
    use super::*;

    /// Marker preserving the reference namespace identity.
    pub struct WorkspaceEvent;

    pub const READY: Definition = define("workspace.ready", None);
    pub const FAILED: Definition = define("workspace.failed", None);
    pub const STATUS: Definition = define("workspace.status", None);

    pub static DEFINITIONS: &[Definition] = &[READY, FAILED, STATUS];
}

/// `packages/schema/src/project.ts` event surface.
pub mod project {
    use super::*;

    pub mod event {
        use super::*;

        pub const UPDATED: Definition = define("project.updated", None);
        pub static DEFINITIONS: &[Definition] = &[UPDATED];
    }
}

/// `packages/schema/src/integration.ts` event surface.
pub mod integration {
    use super::*;

    pub mod event {
        use super::*;

        pub const UPDATED: Definition = define("integration.updated", None);
        pub const CONNECTION_UPDATED: Definition = define("integration.connection.updated", None);
        pub static DEFINITIONS: &[Definition] = &[UPDATED, CONNECTION_UPDATED];
    }
}

/// `packages/schema/src/permission.ts` event surface.
pub mod permission {
    use super::*;

    pub mod event {
        use super::*;

        pub const ASKED: Definition = define("permission.v2.asked", None);
        pub const REPLIED: Definition = define("permission.v2.replied", None);
        pub static DEFINITIONS: &[Definition] = &[ASKED, REPLIED];
    }
}

/// `packages/schema/src/reference.ts` event surface.
pub mod reference {
    use super::*;

    pub mod event {
        use super::*;

        pub const UPDATED: Definition = define("reference.updated", None);
        pub static DEFINITIONS: &[Definition] = &[UPDATED];
    }
}

/// `packages/schema/src/ide-event.ts`.
pub mod ide_event {
    use super::*;

    pub const INSTALLED: Definition = define("ide.installed", None);
    pub static DEFINITIONS: &[Definition] = &[INSTALLED];
}

/// `packages/schema/src/session-todo.ts`.
pub mod session_todo {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// A single todo item. Status and priority are arbitrary strings.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Info {
        pub content: String,
        pub status: String,
        pub priority: String,
    }

    pub mod event {
        use super::*;

        pub const UPDATED: Definition = define("todo.updated", None);
        pub static DEFINITIONS: &[Definition] = &[UPDATED];
    }
}

/// `packages/schema/src/filesystem.ts`.
pub mod filesystem {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// The kind of a filesystem entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum EntryType {
        File,
        Directory,
    }

    /// Input to the filesystem `find` operation.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FindInput {
        pub query: String,
        #[serde(rename = "type", skip_serializing_if = "Option::is_none", default)]
        pub kind: Option<EntryType>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub limit: Option<u32>,
    }

    impl FindInput {
        /// Construct a find input for `query`, leaving the optional filters unset.
        pub fn new(query: impl Into<String>) -> Self {
            Self {
                query: query.into(),
                kind: None,
                limit: None,
            }
        }
    }

    pub mod event {
        use super::*;

        pub const EDITED: Definition = define("file.edited", None);
        pub static DEFINITIONS: &[Definition] = &[EDITED];
    }
}

/// `packages/schema/src/session.ts` compatibility aliases.
pub mod session {
    pub use super::session_event::{SessionEvent as Event, DEFINITIONS as EVENT_DEFINITIONS};
}

/// `packages/schema/src/workspace.ts` compatibility aliases.
pub mod workspace {
    pub use super::workspace_event::{WorkspaceEvent as Event, DEFINITIONS as EVENT_DEFINITIONS};
}
