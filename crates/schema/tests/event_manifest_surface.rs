//! Port of packages/schema/test/event-manifest.test.ts (upstream 18ef3cc).
//!
//! NOTE: the reference file is **stale and unwired** at this commit — it asserts
//! 55/85/85/32 but the source composes 58/88/88/35 (and `packages/opencode/test/
//! event-manifest.test.ts` plus the OpenAPI `V2Event` fixture both pin 88). We
//! assert the authoritative values, keeping the semantic assertions intact.
//!
//! The Rust registry is not implemented yet, so these tests are red (ignored)
//! until it lands.

use opencode_schema::{
    event_manifest, filesystem, ide_event, integration, permission, project, reference, session,
    session_event, session_todo, session_v1, workspace, workspace_event,
};

#[test]
fn owns_the_complete_public_event_surface() {
    assert_eq!(event_manifest::server_definitions().unwrap().len(), 58);
    assert_eq!(event_manifest::definitions().unwrap().len(), 88);

    let definitions = session_v1::event::DEFINITIONS;
    assert_eq!(
        definitions,
        [
            session_v1::event::CREATED,
            session_v1::event::UPDATED,
            session_v1::event::DELETED,
            session_v1::event::MESSAGE_UPDATED,
            session_v1::event::MESSAGE_REMOVED,
            session_v1::event::PART_UPDATED,
            session_v1::event::PART_REMOVED,
            session_v1::event::PART_DELTA,
            session_v1::event::DIFF,
            session_v1::event::ERROR,
        ]
    );

    assert_eq!(event_manifest::latest().unwrap().len(), 88);
    assert_eq!(event_manifest::durable().unwrap().len(), 35);
}

#[test]
fn uses_canonical_definitions_for_current_public_events() {
    assert_eq!(
        std::any::TypeId::of::<session::Event>(),
        std::any::TypeId::of::<session_event::SessionEvent>()
    );
    assert!(std::ptr::eq(
        session::EVENT_DEFINITIONS.as_ptr(),
        session_event::DEFINITIONS.as_ptr()
    ));
    assert_eq!(
        std::any::TypeId::of::<workspace::Event>(),
        std::any::TypeId::of::<workspace_event::WorkspaceEvent>()
    );
    assert!(std::ptr::eq(
        workspace::EVENT_DEFINITIONS.as_ptr(),
        workspace_event::DEFINITIONS.as_ptr()
    ));

    let latest = event_manifest::latest().unwrap();
    assert_eq!(
        **latest.get("session.next.step.ended").unwrap(),
        session_event::step::ENDED
    );
    assert_eq!(
        **latest.get("todo.updated").unwrap(),
        session_todo::event::UPDATED
    );
    assert_eq!(
        **latest.get("project.updated").unwrap(),
        project::event::UPDATED
    );

    assert_eq!(
        project::event::DEFINITIONS.to_vec(),
        vec![project::event::UPDATED]
    );
    assert_eq!(
        filesystem::event::DEFINITIONS.to_vec(),
        vec![filesystem::event::EDITED]
    );
    assert_eq!(
        integration::event::DEFINITIONS.to_vec(),
        vec![
            integration::event::UPDATED,
            integration::event::CONNECTION_UPDATED,
        ]
    );
    assert_eq!(
        permission::event::DEFINITIONS.to_vec(),
        vec![permission::event::ASKED, permission::event::REPLIED]
    );
    assert_eq!(
        reference::event::DEFINITIONS.to_vec(),
        vec![reference::event::UPDATED]
    );

    assert!(!latest.contains_key("ide.installed"));
    assert_eq!(ide_event::DEFINITIONS.to_vec(), vec![ide_event::INSTALLED]);

    let definitions = event_manifest::definitions().unwrap();
    assert_eq!(
        definitions[43..46].to_vec(),
        vec![
            &session_v1::event::PART_DELTA,
            &session_v1::event::DIFF,
            &session_v1::event::ERROR,
        ]
    );

    let durable = event_manifest::durable().unwrap();
    assert!(!durable.contains_key("session.next.step.ended.1"));
    assert_eq!(
        **durable.get("session.next.step.ended.2").unwrap(),
        session_event::step::ENDED
    );
}
