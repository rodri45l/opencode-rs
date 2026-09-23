//! Port of `packages/protocol/test/session-cursor.test.ts`.
//!
//! Keeps the reference's two assertions: a cursor round-trips the full query,
//! and numeric paging inputs are decoded from strings.

use opencode_protocol::{
    decode_cursor, encode_cursor, AnchorDirection, ListAnchor, Order, SessionHistoryQuery,
    SessionsCursorInput,
};
use opencode_schema::{SessionId, WorkspaceId};

fn sample_input() -> SessionsCursorInput {
    SessionsCursorInput {
        workspace: None,
        order: Some(Order::Desc),
        search: Some("protocol".into()),
        directory: None,
        project: None,
        subpath: None,
        anchor: ListAnchor {
            id: SessionId::parse("ses_test").unwrap(),
            time: 1,
            direction: AnchorDirection::Next,
        },
    }
}

#[test]
fn cursor_round_trips() {
    let input = sample_input();
    let cursor = encode_cursor(&input).expect("encode");
    let decoded = decode_cursor(&cursor).expect("decode");
    assert_eq!(decoded, input);
}

#[test]
fn cursor_is_base64url_and_omits_unset_fields() {
    let input = sample_input();
    let cursor = encode_cursor(&input).unwrap();
    assert!(!cursor.contains('+') && !cursor.contains('/') && !cursor.contains('='));

    // Round-trip through the JSON layer to confirm unset scope fields are omitted.
    let json = serde_json::to_value(&input).unwrap();
    assert!(json.get("workspace").is_none());
    assert!(json.get("directory").is_none());
    assert_eq!(json["order"], "desc");
    assert_eq!(json["anchor"]["direction"], "next");
    assert_eq!(json["anchor"]["id"], "ses_test");
}

#[test]
fn workspace_is_preserved_when_present() {
    let mut input = sample_input();
    input.workspace = Some(WorkspaceId::parse("wrk_abc").unwrap());
    let decoded = decode_cursor(&encode_cursor(&input).unwrap()).unwrap();
    assert_eq!(
        decoded.workspace.as_ref().map(|w| w.as_str()),
        Some("wrk_abc")
    );
}

#[test]
fn invalid_cursor_is_rejected() {
    assert!(decode_cursor("not-base64!!").is_err());
    // valid base64url, but not a cursor
    assert!(decode_cursor("aGVsbG8").is_err());
}

#[test]
fn history_query_decodes_numeric_paging_inputs() {
    let query: SessionHistoryQuery = serde_json::from_str(r#"{"after":"3","limit":"10"}"#).unwrap();
    assert_eq!(query.after, Some(3));
    assert_eq!(query.limit, Some(10));
    assert!(query.validate().is_ok());
}

#[test]
fn history_query_enforces_limit_bounds() {
    let too_big: SessionHistoryQuery = serde_json::from_str(r#"{"limit":"101"}"#).unwrap();
    assert!(too_big.validate().is_err());

    let zero: SessionHistoryQuery = serde_json::from_str(r#"{"limit":"0"}"#).unwrap();
    assert!(zero.validate().is_err());

    let accepted: SessionHistoryQuery = serde_json::from_str(r#"{"limit":"100"}"#).unwrap();
    assert!(accepted.validate().is_ok());
}
