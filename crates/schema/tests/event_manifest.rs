//! Contract tests for the generated event surface.
//!
//! These are the first tests written for the port: they pin the Rust types to
//! the fixture extracted from the reference `openapi.json`. If the fixture and
//! the enum ever disagree, this fails.

use opencode_schema::{EventEnvelope, EventType, EVENT_TYPE_COUNT};
use serde_json::Value;

fn fixture() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/events.json"
    );
    let raw = std::fs::read_to_string(path).expect("read events fixture");
    serde_json::from_str(&raw).expect("parse events fixture")
}

#[test]
fn fixture_is_present_and_non_empty() {
    let fx = fixture();
    assert!(!fx["events"].as_array().unwrap().is_empty());
}

#[test]
fn enum_covers_every_fixture_event() {
    let fx = fixture();
    let expected = fx["event_count"].as_u64().unwrap() as usize;
    assert_eq!(
        expected, EVENT_TYPE_COUNT,
        "fixture and enum disagree on count"
    );
    assert_eq!(fx["events"].as_array().unwrap().len(), EVENT_TYPE_COUNT);
}

#[test]
fn every_fixture_event_roundtrips_through_the_enum() {
    let fx = fixture();
    for event in fx["events"].as_array().unwrap() {
        let wire = event["type"].as_str().unwrap();
        let parsed =
            EventType::from_wire(wire).unwrap_or_else(|| panic!("missing variant for {wire}"));
        assert_eq!(parsed.as_str(), wire, "as_str mismatch for {wire}");
        assert_eq!(parsed.to_string(), wire);
    }
}

#[test]
fn serde_uses_the_wire_string() {
    let fx = fixture();
    for event in fx["events"].as_array().unwrap() {
        let wire = event["type"].as_str().unwrap();
        let parsed = EventType::from_wire(wire).unwrap();
        let json = serde_json::to_string(&parsed).unwrap();
        assert_eq!(
            json,
            format!("\"{wire}\""),
            "serialization mismatch for {wire}"
        );
        let back: EventType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, parsed);
    }
}

#[test]
fn envelope_roundtrips_a_minimal_event() {
    let wire = r#"{
        "id": "evt_01890000000070008000000000000000",
        "type": "session.created",
        "data": { "sessionID": "ses_abc" }
    }"#;
    let parsed: EventEnvelope = serde_json::from_str(wire).unwrap();
    assert_eq!(parsed.event_type, EventType::SessionCreated);
    assert_eq!(parsed.name(), "session.created");
    assert!(parsed.durable.is_none());

    let encoded = serde_json::to_value(&parsed).unwrap();
    assert!(
        encoded.get("durable").is_none(),
        "optional fields must be omitted"
    );
    assert_eq!(encoded["type"], "session.created");
}

#[test]
fn envelope_roundtrips_a_durable_event() {
    let wire = r#"{
        "id": "evt_01890000000070008000000000000000",
        "type": "session.next.step.ended",
        "durable": { "aggregateID": "ses_abc", "seq": 3, "version": 2 },
        "location": { "directory": "/tmp/project" },
        "data": { "sessionID": "ses_abc", "finish": "stop" }
    }"#;
    let parsed: EventEnvelope = serde_json::from_str(wire).unwrap();
    let durable = parsed.durable.as_ref().unwrap();
    assert_eq!(durable.aggregate_id, "ses_abc");
    assert_eq!(durable.seq, 3);
    assert_eq!(parsed.location.as_ref().unwrap().directory, "/tmp/project");
}
