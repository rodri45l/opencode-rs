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
    pub fn parse(_patch_text: &str) -> CoreResult<ParsedPatch> {
        Err(CoreError::NotImplemented(
            "tool_apply_patch::ApplyPatchTool::parse",
        ))
    }

    /// The applied-batch summary message.
    pub fn summary(_patch: &ParsedPatch) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_apply_patch::ApplyPatchTool::summary",
        ))
    }

    /// The failure message for a failing hunk.
    pub fn failure_message(_path: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_apply_patch::ApplyPatchTool::failure_message",
        ))
    }
}
