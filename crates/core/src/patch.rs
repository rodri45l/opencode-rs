//! Unified-diff patch parsing and derivation.
//!
//! Ports the observable behaviour of `packages/core/src/patch.ts`: parse
//! add/update/delete hunks (optionally wrapped in a heredoc), derive fuzzy line
//! updates that preserve a BOM, and reject malformed bodies.

use crate::{CoreError, CoreResult};

/// A single change inside an update hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchChunk {
    /// Lines expected in the original file.
    pub old_lines: Vec<String>,
    /// Replacement lines.
    pub new_lines: Vec<String>,
    /// Optional `@@` section label.
    pub change_context: Option<String>,
    /// Whether the chunk anchors to end-of-file.
    pub end_of_file: Option<bool>,
}

/// One parsed hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchHunk {
    /// Create a file with `contents`.
    Add {
        /// Target path.
        path: String,
        /// File contents.
        contents: String,
    },
    /// Update a file.
    Update {
        /// Target path.
        path: String,
        /// Chunks to apply.
        chunks: Vec<PatchChunk>,
        /// Optional move destination.
        move_path: Option<String>,
    },
    /// Delete a file.
    Delete {
        /// Target path.
        path: String,
    },
}

/// Result of deriving an updated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchUpdate {
    /// New file contents (BOM stripped).
    pub content: String,
    /// Whether the original had a BOM.
    pub bom: bool,
}

/// Patch parsing helpers.
#[derive(Debug, Default)]
pub struct Patch;

impl Patch {
    /// Parse a patch body into hunks.
    pub fn parse(_input: &str) -> CoreResult<Vec<PatchHunk>> {
        Err(CoreError::NotImplemented("patch::Patch::parse"))
    }

    /// Derive updated contents by applying `chunks` to `old_content`.
    pub fn derive(
        _path: &str,
        _chunks: Vec<PatchChunk>,
        _old_content: &str,
    ) -> CoreResult<PatchUpdate> {
        Err(CoreError::NotImplemented("patch::Patch::derive"))
    }

    /// Re-attach a BOM to derived contents.
    pub fn join_bom(_content: &str, _bom: bool) -> CoreResult<String> {
        Err(CoreError::NotImplemented("patch::Patch::join_bom"))
    }
}
