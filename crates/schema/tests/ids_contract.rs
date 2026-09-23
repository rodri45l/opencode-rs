//! Contract tests for prefixed identifiers.
//!
//! Pins the id prefixes accepted by the server to the set the reference
//! `openapi.json` declares. `msg` is allowed as an uncovered alias of `msg_`.

use opencode_schema::{
    EventId, MessageId, PartId, PermissionId, PtyId, QuestionId, SessionId, WorkspaceId,
};
use serde_json::Value;

const ALL_ID_PREFIXES: &[&str] = &[
    EventId::PREFIX,
    SessionId::PREFIX,
    MessageId::PREFIX,
    WorkspaceId::PREFIX,
    PermissionId::PREFIX,
    PartId::PREFIX,
    PtyId::PREFIX,
    QuestionId::PREFIX,
];

fn fixture() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/id_prefixes.json"
    );
    let raw = std::fs::read_to_string(path).expect("read id_prefixes fixture");
    serde_json::from_str(&raw).expect("parse id_prefixes fixture")
}

fn fixture_prefixes() -> Vec<String> {
    fixture()["prefixes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn every_id_type_prefix_exists_in_the_contract() {
    let declared = fixture_prefixes();
    for prefix in ALL_ID_PREFIXES {
        assert!(
            declared.iter().any(|d| d == prefix),
            "id prefix {prefix:?} is not declared by the reference contract"
        );
    }
}

#[test]
fn every_contract_prefix_is_covered() {
    // `msg` is a looser alias subsumed by `msg_`.
    const ALLOWED_UNCOVERED: &[&str] = &["msg"];
    let uncovered: Vec<String> = fixture_prefixes()
        .into_iter()
        .filter(|p| !ALL_ID_PREFIXES.contains(&p.as_str()))
        .filter(|p| !ALLOWED_UNCOVERED.contains(&p.as_str()))
        .collect();
    assert!(uncovered.is_empty(), "uncovered id prefixes: {uncovered:?}");
}

#[test]
fn parse_enforces_prefixes() {
    assert!(SessionId::parse("ses_abc").is_ok());
    assert!(SessionId::parse("evt_abc").is_err());
    assert!(MessageId::parse("msg_abc").is_ok());
    assert!(PermissionId::parse("per_abc").is_ok());
    assert!(PartId::parse("prt_abc").is_ok());
    assert!(PtyId::parse("pty_abc").is_ok());
    assert!(QuestionId::parse("que_abc").is_ok());
    assert!(WorkspaceId::parse("wrk_abc").is_ok());
}

#[test]
fn generated_ids_carry_their_prefix() {
    let generated = [
        (EventId::PREFIX, EventId::generate().to_string()),
        (SessionId::PREFIX, SessionId::generate().to_string()),
        (MessageId::PREFIX, MessageId::generate().to_string()),
        (WorkspaceId::PREFIX, WorkspaceId::generate().to_string()),
        (PermissionId::PREFIX, PermissionId::generate().to_string()),
        (PartId::PREFIX, PartId::generate().to_string()),
        (PtyId::PREFIX, PtyId::generate().to_string()),
        (QuestionId::PREFIX, QuestionId::generate().to_string()),
    ];
    for (prefix, id) in generated {
        assert!(id.starts_with(prefix), "{id} should start with {prefix}");
    }
}
