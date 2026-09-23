//! Port of packages/opencode/test/event-manifest.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the public event manifest exposes every latest wire type
//! once (88 latest, 35 durable), maps step/todo events to their canonical
//! definitions, excludes `ide.installed` from latest, and keeps only the
//! current step-settlement version.

use opencode_schema::{event_manifest, session_event, session_todo};

#[test]
#[ignore = "porting: event manifest not implemented"]
fn contains_every_latest_public_wire_type_once() {
    assert_eq!(
        event_manifest::definitions().expect("definitions").len(),
        88
    );
    assert_eq!(event_manifest::latest().expect("latest").len(), 88);
    assert_eq!(event_manifest::durable().expect("durable").len(), 35);

    let latest = event_manifest::latest().expect("latest");
    assert_eq!(
        **latest.get("session.next.step.ended").expect("step.ended"),
        session_event::step::ENDED
    );
    assert_eq!(
        **latest.get("todo.updated").expect("todo.updated"),
        session_todo::event::UPDATED
    );
    assert!(!latest.contains_key("ide.installed"));
    assert!(latest.contains_key("server.connected"));
    assert!(latest.contains_key("global.disposed"));
}

#[test]
#[ignore = "porting: event manifest not implemented"]
fn contains_only_the_current_step_settlement_versions() {
    let durable = event_manifest::durable().expect("durable");
    assert!(!durable.contains_key("session.next.step.ended.1"));
    assert_eq!(
        **durable
            .get("session.next.step.ended.2")
            .expect("step.ended.2"),
        session_event::step::ENDED
    );
}
