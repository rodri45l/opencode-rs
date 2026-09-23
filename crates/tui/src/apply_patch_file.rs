//! Apply-patch file projection.
//!
//! Derived from `packages/session-ui/src/components/apply-patch-file.ts`
//! (upstream 18ef3cc): server patch metadata is parsed into a complete diff
//! view, and legacy before/after payloads keep working.

use std::fmt;

use crate::session_diff;

/// Error raised by the apply-patch projection.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the apply-patch projection.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui apply-patch file projection";

/// One raw patch-file entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchInput {
    pub file_path: String,
    pub relative_path: String,
    pub kind: String,
    pub patch: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub additions: i64,
    pub deletions: i64,
}

/// A resolved file diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub name: String,
    pub is_partial: bool,
    pub deletion_lines: Vec<String>,
    pub addition_lines: Vec<String>,
}

/// A diff view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffView {
    pub file_diff: FileDiff,
}

/// A projected apply-patch file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedFile {
    pub view: DiffView,
}

/// Project raw patch inputs into diff views.
pub fn patch_files(inputs: &[PatchInput]) -> PortResult<Vec<AppliedFile>> {
    Ok(inputs
        .iter()
        .map(|input| {
            let diff = session_diff::resolve_source(
                &input.relative_path,
                input.patch.as_deref(),
                input.before.as_deref(),
                input.after.as_deref(),
            );
            AppliedFile {
                view: DiffView {
                    file_diff: FileDiff {
                        name: diff.name,
                        is_partial: diff.is_partial,
                        deletion_lines: diff.deletion_lines,
                        addition_lines: diff.addition_lines,
                    },
                },
            }
        })
        .collect())
}

/// Render one side of a diff view.
pub fn text(view: &DiffView, side: &str) -> PortResult<String> {
    let lines = if side == "deletions" {
        &view.file_diff.deletion_lines
    } else {
        &view.file_diff.addition_lines
    };
    Ok(lines.join(""))
}
