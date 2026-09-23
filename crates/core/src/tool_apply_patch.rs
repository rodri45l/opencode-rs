//! Apply-patch tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/apply-patch.ts`:
//! the tool is registered as `apply_patch`, parses a `*** Begin Patch` document
//! into sequential add/update/delete operations, rejects moves before applying
//! any hunk (`apply_patch moves are not supported yet`), summarizes an applied
//! batch as `Applied patch sequentially:` followed by `A`/`M`/`D` lines, and
//! reports `Unable to apply patch at <path>` for a failing hunk. The
//! `ToolRegistry`/`FileMutation`/`Permission` wiring, external-directory
//! approval, and interruption semantics are dropped; the pure parsing and
//! formatting remain.

use crate::{CoreError, CoreResult};

/// A parsed patch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOperation {
    /// Add a new file.
    Add {
        /// Target resource path.
        path: String,
    },
    /// Update an existing file.
    Update {
        /// Target resource path.
        path: String,
    },
    /// Delete a file.
    Delete {
        /// Target resource path.
        path: String,
    },
    /// Move an existing file (unsupported).
    Move {
        /// Source resource path.
        from: String,
        /// Destination resource path.
        to: String,
    },
}

/// A parsed patch document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPatch {
    /// The operations in document order.
    pub operations: Vec<PatchOperation>,
}

/// The apply-patch tool.
#[derive(Debug, Default)]
pub struct ApplyPatchTool;

impl ApplyPatchTool {
    /// The tool name.
    pub const NAME: &'static str = "apply_patch";

    /// Error returned when the patch contains a move hunk.
    pub const MOVE_ERROR: &'static str = "apply_patch moves are not supported yet";

    /// Parse a patch document.
    pub fn parse(patch_text: &str) -> CoreResult<ParsedPatch> {
        let hunks = crate::patch::Patch::parse(patch_text)?;
        let mut operations = Vec::new();
        for hunk in hunks {
            match hunk {
                crate::patch::PatchHunk::Add { path, .. } => {
                    operations.push(PatchOperation::Add { path })
                }
                crate::patch::PatchHunk::Update {
                    path, move_path, ..
                } => {
                    if let Some(to) = move_path {
                        operations.push(PatchOperation::Move { from: path, to });
                    } else {
                        operations.push(PatchOperation::Update { path });
                    }
                }
                crate::patch::PatchHunk::Delete { path } => {
                    operations.push(PatchOperation::Delete { path })
                }
            }
        }
        if operations
            .iter()
            .any(|operation| matches!(operation, PatchOperation::Move { .. }))
        {
            return Err(CoreError::Message(Self::MOVE_ERROR.into()));
        }
        Ok(ParsedPatch { operations })
    }

    /// The applied-batch summary message.
    pub fn summary(patch: &ParsedPatch) -> CoreResult<String> {
        let mut lines = vec!["Applied patch sequentially:".to_string()];
        for operation in &patch.operations {
            let (prefix, path) = match operation {
                PatchOperation::Add { path } => ("A", path),
                PatchOperation::Update { path } => ("M", path),
                PatchOperation::Delete { path } => ("D", path),
                PatchOperation::Move { from, .. } => ("M", from),
            };
            lines.push(format!("{prefix} {path}"));
        }
        Ok(lines.join("\n"))
    }

    /// The failure message for a failing hunk.
    pub fn failure_message(path: &str) -> CoreResult<String> {
        Ok(format!("Unable to apply patch at {path}"))
    }
}
