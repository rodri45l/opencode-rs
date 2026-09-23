//! Write tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/write.ts`: the tool
//! is registered as `write`, accepts only `path` and `content`, resolves a path
//! against the active location (absolute external paths stay external and
//! require a `external_directory` approval before `edit`), reports
//! created/wrote messages from the mutation result, and returns
//! `Unable to write <path>` when approval is denied. The `ToolRegistry`,
//! `FileMutation`, and `Permission` service wiring is dropped; the pure
//! decisions remain.

use std::path::{Path, PathBuf};

use crate::{CoreError, CoreResult};

/// A write target resolved against the active location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedWriteTarget {
    /// The canonical absolute target path.
    pub canonical: PathBuf,
    /// The model-facing resource string (forward slashes).
    pub resource: String,
    /// Whether the target lies outside the active location.
    pub external: bool,
}

/// The write tool.
#[derive(Debug, Default)]
pub struct WriteTool;

impl WriteTool {
    /// The tool name.
    pub const NAME: &'static str = "write";

    /// The locked input schema property names.
    pub fn schema_keys() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_write::WriteTool::schema_keys",
        ))
    }

    /// Resolve `path` against the active location directory.
    pub fn resolve(_active_directory: &str, _path: &str) -> CoreResult<ResolvedWriteTarget> {
        Err(CoreError::NotImplemented("tool_write::WriteTool::resolve"))
    }

    /// The success message for a completed write.
    pub fn success_message(_existed: bool, _resource: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_write::WriteTool::success_message",
        ))
    }

    /// The ordered permission actions for a target.
    pub fn permission_actions(_external: bool) -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_write::WriteTool::permission_actions",
        ))
    }

    /// The failure message when a write is denied or fails.
    pub fn failure_message(_path: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_write::WriteTool::failure_message",
        ))
    }

    /// The canonical form of `path` used for the structured `target` field.
    pub fn canonical(_path: &Path) -> CoreResult<PathBuf> {
        Err(CoreError::NotImplemented(
            "tool_write::WriteTool::canonical",
        ))
    }
}
