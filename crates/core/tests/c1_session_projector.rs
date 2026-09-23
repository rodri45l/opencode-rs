//! Port of packages/core/test/session-projector.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a moved Session projects its new directory, staged/cleared/
//! committed reverts transition the persisted revert state and truncate projected
//! messages at the commit boundary, messages paginate in durable aggregate
//! sequence with next/previous cursors, compaction deltas are not projected while
//! ended compactions project their summary/recent, distinct creator events cannot
//! reuse one projected message ID, and only the newest incomplete assistant is
//! the current assistant projection.
//! Re-derived: the Database/EventV2/SQL wiring and the session-row column
//! assertions (`agent`, `model`, `time_updated`) are replaced by an in-memory
//! projection; those persisted-column checks are skipped.

#![allow(dead_code)]

use serde_json::json;

const NOTE: &str = "porting: session projector not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    /// The persisted revert state.
    #[derive(Debug, Clone, PartialEq)]
    pub enum RevertState {
        None,
        Staged {
            message_id: String,
            snapshot: Option<String>,
            files: Vec<String>,
        },
        Committed {
            message_id: String,
        },
    }

    /// A projected message row.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MsgRow {
        pub id: String,
        pub kind: String,
        pub seq: u64,
        pub data: Value,
    }

    /// A projected session.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Projection {
        pub directory: String,
        pub revert: RevertState,
        pub messages: Vec<MsgRow>,
    }

    impl Projection {
        pub fn new(directory: &str) -> Self {
            Self {
                directory: directory.to_string(),
                revert: RevertState::None,
                messages: Vec::new(),
            }
        }

        pub fn apply_moved(&mut self, _directory: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn apply_revert_staged(
            &mut self,
            _message_id: &str,
            _snapshot: Option<&str>,
            _files: Vec<String>,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn apply_revert_cleared(&mut self) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn apply_revert_committed(&mut self, _message_id: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn apply_compaction_delta(
            &mut self,
            _message_id: &str,
            _text: &str,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn apply_compaction_ended(
            &mut self,
            _message_id: &str,
            _summary: &str,
            _recent: &str,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }

        pub fn insert_creator(
            &mut self,
            _id: &str,
            _kind: &str,
            _data: Value,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session projector"))
        }
    }

    /// In-memory assistant projection used by the message updater.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AssistantMemory {
        pub id: String,
        pub completed: bool,
    }

    /// The newest incomplete assistant message index, if any.
    pub fn get_current_assistant(
        _messages: &[AssistantMemory],
    ) -> Result<Option<usize>, PortError> {
        Err(PortError::NotImplemented("session message updater"))
    }

    /// The direction of a pagination cursor.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        Next,
        Previous,
    }

    pub fn paginate(
        _ids: &[String],
        _limit: usize,
        _cursor: Option<(&str, Direction)>,
    ) -> Result<Vec<String>, PortError> {
        Err(PortError::NotImplemented("session message pagination"))
    }
}

use local::{AssistantMemory, Direction, Projection, RevertState};

#[test]
#[ignore = "porting: session projector not implemented"]
fn projects_moved_sessions_without_the_transitional_context_epoch_table() {
    let mut projection = Projection::new("/project");
    projection.apply_moved("/project/subdir").expect(NOTE);
    assert_eq!(projection.directory, "/project/subdir");
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn projects_staged_cleared_and_committed_reverts() {
    let mut projection = Projection::new("/project");
    projection.messages = vec![
        local::MsgRow {
            id: "msg_boundary".to_string(),
            kind: "assistant".to_string(),
            seq: 1,
            data: json!({}),
        },
        local::MsgRow {
            id: "msg_later".to_string(),
            kind: "assistant".to_string(),
            seq: 2,
            data: json!({}),
        },
    ];

    projection
        .apply_revert_staged("msg_boundary", Some("tree"), vec![])
        .expect(NOTE);
    assert!(matches!(
        &projection.revert,
        RevertState::Staged { message_id, snapshot, files }
            if message_id == "msg_boundary" && snapshot.as_deref() == Some("tree") && files.is_empty()
    ));

    projection.apply_revert_cleared().expect(NOTE);
    assert_eq!(projection.revert, RevertState::None);

    projection
        .apply_revert_staged("msg_boundary", None, vec![])
        .expect(NOTE);
    projection
        .apply_revert_committed("msg_boundary")
        .expect(NOTE);
    let ids: Vec<&str> = projection
        .messages
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(ids, vec!["msg_boundary"]);
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn orders_projected_messages_and_context_by_durable_aggregate_sequence() {
    let ids = vec!["msg_first".to_string(), "msg_second".to_string()];

    let first_page = local::paginate(&ids, 1, None).expect(NOTE);
    assert_eq!(first_page, vec!["msg_first".to_string()]);

    let second_page = local::paginate(&ids, 1, Some(("msg_first", Direction::Next))).expect(NOTE);
    assert_eq!(second_page, vec!["msg_second".to_string()]);

    let previous_page =
        local::paginate(&ids, 1, Some(("msg_second", Direction::Previous))).expect(NOTE);
    assert_eq!(previous_page, vec!["msg_first".to_string()]);

    let context = local::paginate(&ids, usize::MAX, None).expect(NOTE);
    assert_eq!(context, ids);
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn projects_durable_context_messages_supported_by_the_updater() {
    let mut projection = Projection::new("/project");
    for kind in ["agent-switched", "model-switched", "synthetic", "shell"] {
        projection
            .insert_creator(&format!("msg_{kind}"), kind, json!({}))
            .expect(NOTE);
    }
    let compaction_id = "msg_compaction";
    projection
        .apply_compaction_delta(compaction_id, "partial")
        .expect(NOTE);
    assert!(!projection
        .messages
        .iter()
        .any(|row| row.kind == "compaction"));

    projection
        .apply_compaction_ended(compaction_id, "summary", "recent context")
        .expect(NOTE);

    let kinds: Vec<&str> = projection
        .messages
        .iter()
        .map(|row| row.kind.as_str())
        .collect();
    assert_eq!(
        kinds,
        vec![
            "agent-switched",
            "model-switched",
            "synthetic",
            "shell",
            "compaction"
        ]
    );
    let compaction = projection
        .messages
        .iter()
        .find(|row| row.kind == "compaction")
        .expect(NOTE);
    assert_eq!(compaction.data["summary"], json!("summary"));
    assert_eq!(compaction.data["recent"], json!("recent context"));
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn rejects_distinct_creator_events_that_reuse_one_projected_message_id() {
    let mut projection = Projection::new("/project");
    let id = "msg_creator_collision";
    projection
        .insert_creator(id, "synthetic", json!({ "text": "keep me" }))
        .expect(NOTE);
    assert!(projection
        .insert_creator(id, "step-started", json!({ "agent": "build" }))
        .is_err());
    assert_eq!(projection.messages.len(), 1);
    assert_eq!(projection.messages[0].kind, "synthetic");
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn does_not_revive_a_stale_incomplete_in_memory_assistant_projection() {
    let stale = AssistantMemory {
        id: "msg_assistant_stale".to_string(),
        completed: false,
    };
    let completed = AssistantMemory {
        id: "msg_assistant_completed".to_string(),
        completed: true,
    };
    assert_eq!(
        local::get_current_assistant(&[stale, completed]).expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: session projector not implemented"]
fn updates_only_the_newest_incomplete_assistant_projection() {
    let first = AssistantMemory {
        id: "msg_assistant_1".to_string(),
        completed: false,
    };
    let second = AssistantMemory {
        id: "msg_assistant_2".to_string(),
        completed: false,
    };
    assert_eq!(
        local::get_current_assistant(&[first, second]).expect(NOTE),
        Some(1)
    );
}
