//! Review diff virtualization threshold.
//!
//! Derived from
//! `packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.ts`
//! (upstream 18ef3cc): a review diff is virtualized only when either side exceeds
//! 500 lines.

use std::fmt;

/// Error raised by the virtualization helper.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the virtualization helper.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui review diff virtualization";

/// The line counts of a review diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffSize {
    pub addition_lines: i64,
    pub deletion_lines: i64,
}

/// Whether the diff exceeds the virtualization threshold.
pub fn should_virtualize_review_diff(size: DiffSize) -> PortResult<bool> {
    Ok(size.addition_lines.max(size.deletion_lines) > 500)
}
