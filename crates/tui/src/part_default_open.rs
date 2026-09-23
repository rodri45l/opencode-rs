//! Default-open rule for tool parts.
//!
//! Derived from `packages/session-ui/src/components/part-default-open.ts`
//! (upstream 18ef3cc).

use std::fmt;

/// Error raised by the default-open helper.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the default-open helper.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui part-default-open";

/// One file entry in an apply-patch part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchFile {
    pub file_path: String,
    pub kind: String,
}

/// Tool metadata relevant to the default-open decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartMetadata {
    Edit { additions: i64, deletions: i64 },
    ApplyPatch { files: Vec<PatchFile> },
    None,
}

/// A tool part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolPart {
    pub tool: String,
    pub metadata: PartMetadata,
}

fn deletion_only(metadata: &PartMetadata) -> bool {
    match metadata {
        PartMetadata::Edit {
            additions,
            deletions,
        } => *additions == 0 && *deletions > 0,
        PartMetadata::ApplyPatch { files } => {
            !files.is_empty() && files.iter().all(|file| file.kind == "delete")
        }
        PartMetadata::None => false,
    }
}

/// Whether a tool part defaults to open.
pub fn part_default_open(
    part: &ToolPart,
    default_open: bool,
    expand_edits: bool,
) -> PortResult<bool> {
    if part.tool == "bash" || part.tool == "shell" {
        return Ok(default_open);
    }
    if matches!(
        part.tool.as_str(),
        "edit" | "write" | "patch" | "apply_patch"
    ) {
        if !expand_edits {
            return Ok(false);
        }
        return Ok(!deletion_only(&part.metadata));
    }
    Ok(false)
}
