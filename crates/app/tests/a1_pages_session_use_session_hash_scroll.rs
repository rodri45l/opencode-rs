//! Port of packages/app/src/pages/session/use-session-hash-scroll.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

// Local stub (fast wave): real module lands later.
fn message_id_from_hash(_hash: &str) -> Option<String> {
    None
}

#[test]
#[ignore = "porting: pages/session/use-session-hash-scroll not implemented"]
fn parses_hash_with_leading_hash() {
    assert_eq!(
        message_id_from_hash("#message-abc123"),
        Some("abc123".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/use-session-hash-scroll not implemented"]
fn parses_raw_hash_fragment() {
    assert_eq!(message_id_from_hash("message-42"), Some("42".to_string()));
}

#[test]
#[ignore = "porting: pages/session/use-session-hash-scroll not implemented"]
fn ignores_non_message_anchors() {
    assert_eq!(message_id_from_hash("#review-panel"), None);
}
