//! Port of packages/core/test/event.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: publishing tags the envelope with the bound location and
//! the definition's durable version, omits location otherwise, and `latest`
//! selects the newest durable definition per type regardless of order. Dropped
//! (re-derived): the database-backed replay, owner-claim, sequence, projector,
//! and bounded-subscriber cases, which depend on SQLite and Effect fibers.

use opencode_core::event::{DurableDef, EventDefinition, EventLocation, EventService};
use serde_json::json;

const NOTE: &str = "porting: event not implemented";

fn test_location() -> EventLocation {
    EventLocation {
        directory: "project".into(),
        workspace_id: Some("wrk_test".into()),
    }
}

#[test]
#[ignore = "porting: event not implemented"]
fn publishes_events_with_the_current_location() {
    let events = EventService::new(Some(test_location()));
    let message = EventDefinition::new("test.message", None);

    let event = events
        .publish(&message, json!({ "text": "hello" }))
        .expect(NOTE);

    assert_eq!(event.event_type, "test.message");
    assert!(event.durable.is_none());
    assert_eq!(event.data, json!({ "text": "hello" }));
    assert_eq!(event.location, Some(test_location()));
}

#[test]
#[ignore = "porting: event not implemented"]
fn omits_location_when_no_location_is_available() {
    let events = EventService::new(None);
    let message = EventDefinition::new("test.global", None);

    let event = events
        .publish(&message, json!({ "text": "hello" }))
        .expect(NOTE);

    assert!(event.location.is_none());
    assert_eq!(event.event_type, "test.global");
}

#[test]
#[ignore = "porting: event not implemented"]
fn publishes_the_definition_version() {
    let events = EventService::new(Some(test_location()));
    let versioned = EventDefinition::new(
        "test.versioned",
        Some(DurableDef {
            version: 2,
            aggregate: "id".into(),
        }),
    );

    let event = events
        .publish(&versioned, json!({ "id": "one", "text": "hello" }))
        .expect(NOTE);

    assert_eq!(event.event_type, "test.versioned");
    assert_eq!(event.durable.map(|durable| durable.version), Some(2));
}

#[test]
#[ignore = "porting: event not implemented"]
fn selects_the_latest_durable_definition_independent_of_declaration_order() {
    let latest = EventDefinition::new(
        "test.out-of-order",
        Some(DurableDef {
            version: 2,
            aggregate: "id".into(),
        }),
    );
    let historical = EventDefinition::new(
        "test.out-of-order",
        Some(DurableDef {
            version: 1,
            aggregate: "id".into(),
        }),
    );

    let forward = EventService::latest(&[latest.clone(), historical.clone()]).expect(NOTE);
    assert_eq!(forward.get("test.out-of-order"), Some(&latest));

    let reverse = EventService::latest(&[historical, latest.clone()]).expect(NOTE);
    assert_eq!(reverse.get("test.out-of-order"), Some(&latest));
}
