//! Port of packages/tui/test/editor.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/editor.ts; see docs/TEST-PORT.md.
//! Launching an external editor is process/runtime behaviour (n/a, human-verified).

use opencode_tui::editor::normalize_prompt_content;

#[test]
fn normalizes_a_single_trailing_editor_newline_for_one_line_prompts() {
    assert_eq!(normalize_prompt_content("hello\n"), "hello");
    assert_eq!(normalize_prompt_content("hello\r\n"), "hello");
}

#[test]
fn preserves_multiline_prompts_that_end_with_a_newline() {
    assert_eq!(normalize_prompt_content("hello\nworld\n"), "hello\nworld\n");
}
