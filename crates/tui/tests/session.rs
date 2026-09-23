//! Port of packages/tui/test/util/session.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/session.ts; see docs/TEST-PORT.md.

use opencode_tui::session_util::is_default_title;

#[test]
fn recognizes_generated_parent_and_child_titles() {
    assert!(is_default_title("New session - 2026-06-06T12:34:56.789Z"));
    assert!(is_default_title("Child session - 2026-06-06T12:34:56.789Z"));
    assert!(!is_default_title("New session - custom"));
}
