//! Port of packages/client/test/contract-identity.test.ts (upstream 18ef3cc).
//!
//! The reference pins that Core/Server reuse the authoritative Schema and
//! Protocol values and that the client and server contracts generate
//! identically. Those assertions need `opencode-protocol`, `opencode-core`, and
//! `opencode-server`; adding them to `opencode-client` is out of scope (no new
//! dependencies), so the codegen-identity and Project/Provider id assertions are
//! skipped here with this note. The identity assertions that only need the
//! schema crate are ported directly.

use opencode_schema::{EventEnvelope, SessionId, WorkspaceId};

#[test]
fn client_identity_reuses_authoritative_prefixed_ids() {
    assert!(SessionId::create().as_str().starts_with("ses_"));
    assert!(WorkspaceId::create().as_str().starts_with("wrk_"));
}

#[test]
fn shared_dto_schemas_decode_plain_json_objects() {
    let envelope: EventEnvelope =
        serde_json::from_str(r#"{"id":"evt_1","type":"server.connected","data":{}}"#)
            .expect("decode envelope");
    assert_eq!(envelope.name(), "server.connected");
    assert_eq!(envelope.data, serde_json::json!({}));
}
