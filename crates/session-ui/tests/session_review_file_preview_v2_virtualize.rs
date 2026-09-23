//! Port of
//! packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.test.ts
//! (upstream 18ef3cc). Behaviour pinned by the virtualize helper; see docs/TEST-PORT.md.

use opencode_session_ui::session_review_file_preview_v2_virtualize::should_virtualize_review_diff;

#[test]
fn renders_small_diffs_directly() {
    assert!(!should_virtualize_review_diff(500, 500));
}

#[test]
fn virtualizes_large_diffs() {
    assert!(should_virtualize_review_diff(501, 1));
    assert!(should_virtualize_review_diff(1, 501));
}
