//! Port of packages/session-ui/src/components/message-part.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-part-text.ts:
//! `readPartText` prefers a trimmed accumulator hit, otherwise trims the part text,
//! returning an empty string for missing or whitespace-only text.
//! Red-first: the part-text reader is not implemented.

#[allow(dead_code)]
mod part_text {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: session-ui message part text not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PartText {
        pub id: String,
        pub text: Option<String>,
    }

    pub fn read_part_text(
        _accum: Option<&BTreeMap<String, String>>,
        _part: &PartText,
    ) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }
}

use part_text::{read_part_text, PartText, NOTE};

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
#[ignore = "porting: session-ui message part text not implemented"]
fn returns_empty_string_when_accum_is_undefined_and_part_text_is_undefined() {
    assert_eq!(read_part_text(None, &part("part_1", None)).expect(NOTE), "");
}

#[test]
#[ignore = "porting: session-ui message part text not implemented"]
fn returns_trimmed_part_text_when_accum_is_undefined() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("  hello  "))).expect(NOTE),
        "hello"
    );
}

#[test]
#[ignore = "porting: session-ui message part text not implemented"]
fn prefers_accum_value_over_part_text_when_accum_has_a_hit() {
    let store = accum(&[("part_1", "  from accum  ")]);
    assert_eq!(
        read_part_text(Some(&store), &part("part_1", Some("from part"))).expect(NOTE),
        "from accum"
    );
}

#[test]
#[ignore = "porting: session-ui message part text not implemented"]
fn falls_back_to_part_text_when_accum_misses() {
    let store = accum(&[("other_part", "ignored")]);
    assert_eq!(
        read_part_text(Some(&store), &part("part_1", Some("  from part  "))).expect(NOTE),
        "from part"
    );
}

#[test]
#[ignore = "porting: session-ui message part text not implemented"]
fn returns_empty_string_for_whitespace_only_text() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("   \n\t  "))).expect(NOTE),
        ""
    );
}

#[test]
#[ignore = "porting: session-ui message part text not implemented"]
fn trims_leading_and_trailing_whitespace() {
    assert_eq!(
        read_part_text(None, &part("part_1", Some("\n  body  \n"))).expect(NOTE),
        "body"
    );
}
