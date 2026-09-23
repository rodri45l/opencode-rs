//! Port of packages/ui/src/context/marked-regression.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `marked` inline pipeline; see docs/TEST-PORT.md.

use opencode_ui::markdown::render_markdown;

#[test]
fn preserves_code_spans_adjacent_to_tildes() {
    assert_eq!(
        render_markdown("~`0.1576` to measurement-window-only `0.00092`"),
        "<p>~<code>0.1576</code> to measurement-window-only <code>0.00092</code></p>\n"
    );
    assert_eq!(
        render_markdown("`before`~`after`"),
        "<p><code>before</code>~<code>after</code></p>\n"
    );
    assert_eq!(
        render_markdown("~~`deleted code`~~"),
        "<p><del><code>deleted code</code></del></p>\n"
    );
}
