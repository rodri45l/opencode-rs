//! Review diff virtualization threshold.
//!
//! Port of
//! packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.ts
//! behaviour (upstream 18ef3cc).

/// Whether a review diff is large enough to virtualize.
pub fn should_virtualize_review_diff(addition_lines: i64, deletion_lines: i64) -> bool {
    addition_lines > 500 || deletion_lines > 500
}
