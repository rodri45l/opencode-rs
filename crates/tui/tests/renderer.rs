//! Port of packages/tui/test/util/renderer.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/renderer.ts; see docs/TEST-PORT.md.

use opencode_tui::renderer::destroy_renderer;
use std::cell::RefCell;

#[test]
fn clears_the_terminal_title_before_destroying_the_renderer() {
    let calls = RefCell::new(Vec::new());
    destroy_renderer(
        false,
        |title| calls.borrow_mut().push(format!("title:{title}")),
        || calls.borrow_mut().push("destroy".to_string()),
    );
    assert_eq!(*calls.borrow(), vec!["title:", "destroy"]);
}

#[test]
fn still_clears_the_title_after_renderer_destruction() {
    let calls = RefCell::new(Vec::new());
    destroy_renderer(
        true,
        |title| calls.borrow_mut().push(format!("title:{title}")),
        || calls.borrow_mut().push("destroy".to_string()),
    );
    assert_eq!(*calls.borrow(), vec!["title:"]);
}
