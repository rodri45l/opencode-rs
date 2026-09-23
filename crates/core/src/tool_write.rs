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

use crate::CoreResult;

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
        Ok(vec!["path", "content"])
    }

    /// Resolve `path` against the active location directory.
    pub fn resolve(active_directory: &str, path: &str) -> CoreResult<ResolvedWriteTarget> {
        let base = Path::new(active_directory);
        let joined = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            base.join(path)
        };
        let canonical = Self::canonical(&joined)?;
        let external = !canonical.starts_with(base);
        let resource = canonical
            .strip_prefix(base)
            .unwrap_or(&canonical)
            .to_string_lossy()
            .replace('\\', "/");
        Ok(ResolvedWriteTarget {
            canonical,
            resource,
            external,
        })
    }

    /// The success message for a completed write.
    pub fn success_message(existed: bool, resource: &str) -> CoreResult<String> {
        let verb = if existed { "Wrote" } else { "Created" };
        Ok(format!("{verb} file successfully: {resource}"))
    }

    /// The ordered permission actions for a target.
    pub fn permission_actions(external: bool) -> CoreResult<Vec<&'static str>> {
        if external {
            Ok(vec!["external_directory", "edit"])
        } else {
            Ok(vec!["edit"])
        }
    }

    /// The failure message when a write is denied or fails.
    pub fn failure_message(path: &str) -> CoreResult<String> {
        Ok(format!("Unable to write {path}"))
    }

    /// The canonical form of `path` used for the structured `target` field.
    pub fn canonical(path: &Path) -> CoreResult<PathBuf> {
        // Resolve `.`/`..` lexically without requiring the file to exist.
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                std::path::Component::CurDir => {}
                other => normalized.push(other.as_os_str()),
            }
        }
        Ok(normalized)
    }
}
