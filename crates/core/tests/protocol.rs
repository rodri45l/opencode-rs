//! Port of packages/core/test/pty/protocol.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the protocol drops invalid binary input frames while
//! decoding valid text and byte input, encodes the cursor as a `0x00`-prefixed
//! JSON control frame, and splits replays into bounded frames.

use opencode_core::pty::PtyProtocol;
use serde_json::json;

const NOTE: &str = "porting: pty protocol not implemented";

#[test]
#[ignore = "porting: pty protocol not implemented"]
fn drops_invalid_binary_input_frames_and_decodes_valid_ones() {
    assert_eq!(
        PtyProtocol::decode_input_str("ready").expect(NOTE),
        Some("ready".into())
    );
    assert_eq!(
        PtyProtocol::decode_input_bytes(&[0xff, 0xfe, 0xfd]).expect(NOTE),
        None
    );
    assert_eq!(
        PtyProtocol::decode_input_bytes(b"hello").expect(NOTE),
        Some("hello".into())
    );
}

#[test]
#[ignore = "porting: pty protocol not implemented"]
fn encodes_the_cursor_as_a_zero_prefixed_json_control_frame() {
    let frame = PtyProtocol::meta_frame(42).expect(NOTE);
    assert_eq!(frame[0], 0);

    let decoded: serde_json::Value = serde_json::from_slice(&frame[1..]).expect(NOTE);
    assert_eq!(decoded, json!({ "cursor": 42 }));
}

#[test]
#[ignore = "porting: pty protocol not implemented"]
fn splits_replay_into_bounded_frames() {
    assert_eq!(PtyProtocol::chunks("").expect(NOTE), Vec::<String>::new());
    assert_eq!(
        PtyProtocol::chunks("abc").expect(NOTE),
        vec!["abc".to_string()]
    );

    let big = "x".repeat(PtyProtocol::REPLAY_CHUNK + 1);
    let frames = PtyProtocol::chunks(&big).expect(NOTE);
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].len(), PtyProtocol::REPLAY_CHUNK);
    assert_eq!(frames.concat(), big);
}
