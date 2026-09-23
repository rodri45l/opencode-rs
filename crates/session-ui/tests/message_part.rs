//! Port of packages/session-ui/src/components/message-part.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-part-text.ts; see docs/TEST-PORT.md.

use opencode_session_ui::message_part_text::read_part_text;
use std::collections::HashMap;

#[test]
fn returns_empty_string_when_accum_is_undefined_and_part_text_is_undefined() {
    assert_eq!(read_part_text(None, "part_1", None), "");
}

#[test]
fn returns_trimmed_part_text_when_accum_is_undefined() {
    assert_eq!(read_part_text(None, "part_1", Some("  hello  ")), "hello");
}

#[test]
fn prefers_accum_value_over_part_text_when_accum_has_a_hit() {
    let mut accum = HashMap::new();
    accum.insert("part_1".to_string(), "  from accum  ".to_string());
    assert_eq!(
        read_part_text(Some(&accum), "part_1", Some("from part")),
        "from accum"
    );
}

#[test]
fn falls_back_to_part_text_when_accum_misses() {
    let mut accum = HashMap::new();
    accum.insert("other_part".to_string(), "ignored".to_string());
    assert_eq!(
        read_part_text(Some(&accum), "part_1", Some("  from part  ")),
        "from part"
    );
}

#[test]
fn returns_empty_string_for_whitespace_only_text() {
    assert_eq!(read_part_text(None, "part_1", Some("   \n\t  ")), "");
}

#[test]
fn trims_leading_and_trailing_whitespace() {
    assert_eq!(read_part_text(None, "part_1", Some("\n  body  \n")), "body");
}
