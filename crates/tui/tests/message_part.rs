//! Port of packages/session-ui/src/components/message-part.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-part-text.ts:
//! `readPartText` prefers a trimmed accumulator hit, otherwise trims the part text,
//! returning an empty string for missing or whitespace-only text.
//! Red-first: the part-text reader is not implemented.

use opencode_tui::message_part_text::{read_part_text, PartText, NOTE};

fn part(id: &str, text: Option<&str>) -> PartText {
    PartText {
        id: id.into(),
        text: text.map(str::to_string),
    }
}

fn accum(entries: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn returns_empty_string_when_accum_is_undefined_and_part_text_is_undefined() {
    assert_eq!(read_part_text(None, &part("part_1", None)).expect(NOTE), "");
}

#[test]
fn returns_trimmed_part_text_when_accum_is_undefined() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("  hello  "))).expect(NOTE),
        "hello"
    );
}

#[test]
fn prefers_accum_value_over_part_text_when_accum_has_a_hit() {
    let store = accum(&[("part_1", "  from accum  ")]);
    assert_eq!(
        read_part_text(Some(&store), &part("part_1", Some("from part"))).expect(NOTE),
        "from accum"
    );
}

#[test]
fn falls_back_to_part_text_when_accum_misses() {
    let store = accum(&[("other_part", "ignored")]);
    assert_eq!(
        read_part_text(Some(&store), &part("part_1", Some("  from part  "))).expect(NOTE),
        "from part"
    );
}

#[test]
fn returns_empty_string_for_whitespace_only_text() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("   \n\t  "))).expect(NOTE),
        ""
    );
}

#[test]
fn trims_leading_and_trailing_whitespace() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("\n  body  \n"))).expect(NOTE),
        "body"
    );
}
