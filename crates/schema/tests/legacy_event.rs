//! Port of packages/schema/test/legacy-event.test.ts (upstream 18ef3cc).

use opencode_schema::{legacy_event, permission_v1, project, question_v1, session_v1};

#[test]
fn owns_all_session_v1_definitions() {
    let types: Vec<&str> = session_v1::event::DEFINITIONS
        .iter()
        .map(|definition| definition.event_type)
        .collect();
    assert_eq!(
        types,
        vec![
            "session.created",
            "session.updated",
            "session.deleted",
            "message.updated",
            "message.removed",
            "message.part.updated",
            "message.part.removed",
            "message.part.delta",
            "session.diff",
            "session.error",
        ]
    );

    let durable: Vec<_> = session_v1::event::DEFINITIONS
        .iter()
        .filter(|definition| definition.durable.is_some())
        .collect();
    assert_eq!(durable.len(), 7);
    assert!(durable
        .iter()
        .all(|definition| definition.durable.unwrap().aggregate == "sessionID"));
    assert!(durable
        .iter()
        .all(|definition| definition.durable.unwrap().version == 1));
}

#[test]
fn owns_the_legacy_transient_public_definitions() {
    let actual = [
        session_v1::PART_DELTA.event_type,
        session_v1::DIFF.event_type,
        session_v1::ERROR.event_type,
        permission_v1::event::ASKED.event_type,
        permission_v1::event::REPLIED.event_type,
        question_v1::event::ASKED.event_type,
        question_v1::event::REPLIED.event_type,
        question_v1::event::REJECTED.event_type,
        project::event::UPDATED.event_type,
        legacy_event::COMMAND_EXECUTED.event_type,
    ];
    assert_eq!(
        actual,
        [
            "message.part.delta",
            "session.diff",
            "session.error",
            "permission.asked",
            "permission.replied",
            "question.asked",
            "question.replied",
            "question.rejected",
            "project.updated",
            "command.executed",
        ]
    );
}
