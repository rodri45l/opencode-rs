//! Port of packages/tui/test/prompt/display.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/prompt/display.ts; see docs/TEST-PORT.md.

use opencode_tui::prompt_display::{
    display_char_at, display_slice, mention_trigger_index, prompt_offset_width,
};

#[test]
fn uses_display_width_offsets_for_mentions() {
    assert_eq!(mention_trigger_index("@", None), Some(0));
    assert_eq!(mention_trigger_index("test @", None), Some(5));
    assert_eq!(mention_trigger_index("中文 @", None), Some(5));
    assert_eq!(mention_trigger_index("こんにちは @", None), Some(11));
    assert_eq!(mention_trigger_index("한국어 @", None), Some(7));
    assert_eq!(mention_trigger_index("🙂 @", None), Some(3));
    assert_eq!(
        mention_trigger_index("中文 @src file", Some(prompt_offset_width("中文 @src"))),
        Some(5)
    );
    assert_eq!(
        display_char_at("中文 @src", prompt_offset_width("中文 @")),
        Some("s")
    );
    assert_eq!(
        display_slice("中文 @src", 5, prompt_offset_width("中文 @src")),
        "@src"
    );
    assert_eq!(
        display_slice("中文 @src", 6, prompt_offset_width("中文 @src")),
        "src"
    );
    assert_eq!(
        mention_trigger_index("👨‍👩‍👧‍👦 @src", Some(prompt_offset_width("👨‍👩‍👧‍👦 @src"))),
        Some(3)
    );
    assert_eq!(
        display_char_at("👨‍👩‍👧‍👦 @src", prompt_offset_width("👨‍👩‍👧‍👦 @")),
        Some("s")
    );
    assert_eq!(
        display_slice("👨‍👩‍👧‍👦 @src", 3, prompt_offset_width("👨‍👩‍👧‍👦 @src")),
        "@src"
    );
    assert_eq!(mention_trigger_index("@file1\n@file2", Some(13)), Some(7));
    assert_eq!(display_char_at("@file1\n@file2", 6), Some("\n"));
    assert_eq!(display_slice("@file1\n@file2", 8, 13), "file2");
    assert_eq!(
        mention_trigger_index("@file1\nfoo @file2", Some(17)),
        Some(11)
    );
    assert_eq!(mention_trigger_index("中文 @one\n@two", Some(14)), Some(10));
    assert_eq!(display_slice("中文 @one\n@two", 11, 14), "two");
}

#[test]
fn ignores_mentions_without_a_whitespace_boundary_or_with_trailing_content() {
    assert_eq!(mention_trigger_index("中文@", None), None);
    assert_eq!(mention_trigger_index("こんにちは@", None), None);
    assert_eq!(mention_trigger_index("한국어@", None), None);
    assert_eq!(mention_trigger_index("🙂@", None), None);
    assert_eq!(mention_trigger_index("hello@", None), None);
    assert_eq!(mention_trigger_index("foo@bar.com", None), None);
    assert_eq!(mention_trigger_index("中文 @src file", None), None);
}
