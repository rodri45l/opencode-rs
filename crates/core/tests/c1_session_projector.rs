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
//! projection in `opencode_core::session_projector`.

use opencode_core::session_projector::{
    get_current_assistant, paginate, AssistantMemory, Direction, MsgRow, Projection, RevertState,
};
use serde_json::json;

#[test]
fn projects_moved_sessions_without_the_transitional_context_epoch_table() {
    let mut projection = Projection::new("/project");
    projection.apply_moved("/project/subdir");
    assert_eq!(projection.directory, "/project/subdir");
}

#[test]
fn projects_staged_cleared_and_committed_reverts() {
    let mut projection = Projection::new("/project");
    projection.messages = vec![
        MsgRow {
            id: "msg_boundary".to_string(),
            kind: "assistant".to_string(),
            seq: 1,
            data: json!({}),
        },
        MsgRow {
            id: "msg_later".to_string(),
            kind: "assistant".to_string(),
            seq: 2,
            data: json!({}),
        },
    ];

    projection.apply_revert_staged("msg_boundary", Some("tree"), vec![]);
    assert!(matches!(
        &projection.revert,
        RevertState::Staged { message_id, snapshot, files }
            if message_id == "msg_boundary" && snapshot.as_deref() == Some("tree") && files.is_empty()
    ));

    projection.apply_revert_cleared();
    assert_eq!(projection.revert, RevertState::None);

    projection.apply_revert_staged("msg_boundary", None, vec![]);
    projection.apply_revert_committed("msg_boundary");
    let ids: Vec<&str> = projection
        .messages
        .iter()
        .map(|row| row.id.as_str())
        .collect();
    assert_eq!(ids, vec!["msg_boundary"]);
}

#[test]
fn orders_projected_messages_and_context_by_durable_aggregate_sequence() {
    let ids = vec!["msg_first".to_string(), "msg_second".to_string()];

    let first_page = paginate(&ids, 1, None);
    assert_eq!(first_page, vec!["msg_first".to_string()]);

    let second_page = paginate(&ids, 1, Some(("msg_first", Direction::Next)));
    assert_eq!(second_page, vec!["msg_second".to_string()]);

    let previous_page = paginate(&ids, 1, Some(("msg_second", Direction::Previous)));
    assert_eq!(previous_page, vec!["msg_first".to_string()]);

    let context = paginate(&ids, usize::MAX, None);
    assert_eq!(context, ids);
}

#[test]
fn projects_durable_context_messages_supported_by_the_updater() {
    let mut projection = Projection::new("/project");
    for kind in ["agent-switched", "model-switched", "synthetic", "shell"] {
        projection
            .insert_creator(&format!("msg_{kind}"), kind, json!({}))
            .expect("insert creator");
    }
    let compaction_id = "msg_compaction";
    projection.apply_compaction_delta(compaction_id, "partial");
    assert!(!projection
        .messages
        .iter()
        .any(|row| row.kind == "compaction"));

    projection.apply_compaction_ended(compaction_id, "summary", "recent context");

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
        .expect("compaction");
    assert_eq!(compaction.data["summary"], json!("summary"));
    assert_eq!(compaction.data["recent"], json!("recent context"));
}

#[test]
fn rejects_distinct_creator_events_that_reuse_one_projected_message_id() {
    let mut projection = Projection::new("/project");
    let id = "msg_creator_collision";
    projection
        .insert_creator(id, "synthetic", json!({ "text": "keep me" }))
        .expect("insert");
    assert!(projection
        .insert_creator(id, "step-started", json!({ "agent": "build" }))
        .is_err());
    assert_eq!(projection.messages.len(), 1);
    assert_eq!(projection.messages[0].kind, "synthetic");
}

#[test]
fn does_not_revive_a_stale_incomplete_in_memory_assistant_projection() {
    let stale = AssistantMemory {
        id: "msg_assistant_stale".to_string(),
        completed: false,
    };
    let completed = AssistantMemory {
        id: "msg_assistant_completed".to_string(),
        completed: true,
    };
    assert_eq!(get_current_assistant(&[stale, completed]), None);
}

#[test]
fn updates_only_the_newest_incomplete_assistant_projection() {
    let first = AssistantMemory {
        id: "msg_assistant_1".to_string(),
        completed: false,
    };
    let second = AssistantMemory {
        id: "msg_assistant_2".to_string(),
        completed: false,
    };
    assert_eq!(get_current_assistant(&[first, second]), Some(1));
}
