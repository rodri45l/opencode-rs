//! Port of packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/v2/components/session-review-file-preview-v2-virtualize.ts:
//! a review diff is virtualized only when either side exceeds 500 lines.
//! Red-first: the virtualization threshold is not implemented.

#[allow(dead_code)]
mod virtualize {
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

    pub const NOTE: &str = "porting: session-ui review diff virtualization not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DiffSize {
        pub addition_lines: i64,
        pub deletion_lines: i64,
    }

    pub fn should_virtualize_review_diff(_size: DiffSize) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use virtualize::{should_virtualize_review_diff, DiffSize, NOTE};

#[test]
#[ignore = "porting: session-ui review diff virtualization not implemented"]
fn renders_small_diffs_directly() {
    assert!(!should_virtualize_review_diff(DiffSize {
        addition_lines: 500,
        deletion_lines: 500,
    })
    .expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui review diff virtualization not implemented"]
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
