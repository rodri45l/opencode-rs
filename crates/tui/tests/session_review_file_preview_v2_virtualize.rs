//! Port of packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.ts:
//! a review diff is virtualized only when either side exceeds 500 lines.
//! Red-first: the virtualization threshold is not implemented.

use opencode_tui::session_review_file_preview_v2_virtualize::{
    should_virtualize_review_diff, DiffSize, NOTE,
};

#[test]
fn renders_small_diffs_directly() {
    assert!(!should_virtualize_review_diff(DiffSize {
        addition_lines: 500,
        deletion_lines: 500,
    })
    .expect(NOTE));
}

#[test]
fn virtualizes_large_diffs() {
    assert!(should_virtualize_review_diff(DiffSize {
        addition_lines: 501,
        deletion_lines: 1,
    })
    .expect(NOTE));
    assert!(should_virtualize_review_diff(DiffSize {
        addition_lines: 1,
        deletion_lines: 501,
    })
    .expect(NOTE));
}
