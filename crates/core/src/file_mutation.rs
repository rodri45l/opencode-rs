//! Guarded file mutations.
//!
//! Ports the observable behaviour of `packages/core/src/file-mutation.ts` and
//! `location-mutation.ts`: resolved targets carry a canonical path and a
//! resource name, writes report whether the target existed, conditional
//! creates/writes fail on appearing or stale targets, and text writes preserve
//! exactly one BOM.

use std::path::{Path, PathBuf};

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A resolved mutation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    /// Canonical absolute path.
    pub canonical: AbsolutePath,
    /// Resource name relative to the location (or the path for externals).
    pub resource: String,
}

/// Resolves paths into mutation targets.
#[derive(Debug, Default)]
pub struct LocationMutation {
    directory: PathBuf,
}

impl LocationMutation {
    /// Bind a mutation resolver to a location directory.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// Resolve `path` against the active location.
    pub fn resolve(&self, path: &str) -> CoreResult<ResolvedTarget> {
        let base = crate::location_mutation::LocationMutation::resolve(
            &AbsolutePath::new(self.directory.clone()),
            &crate::location_mutation::ResolveInput {
                path: path.to_string(),
                kind: None,
            },
        )?;
        Ok(ResolvedTarget {
            canonical: base.canonical,
            resource: base.resource,
        })
    }
}

/// The outcome of a file mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMutationResult {
    /// `write` or `remove`.
    pub operation: String,
    /// Canonical target path.
    pub target: AbsolutePath,
    /// Resource name.
    pub resource: String,
    /// Whether the target existed before the mutation.
    pub existed: bool,
}

/// Guarded file mutations.
#[derive(Debug, Default)]
pub struct FileMutation;

impl FileMutation {
    /// Write `content` to `target`.
    pub fn write(&self, target: &ResolvedTarget, content: &str) -> CoreResult<FileMutationResult> {
        self.write_bytes(target, content.as_bytes())
    }

    /// Create `target`, failing if it appeared after resolution.
    pub fn create(&self, target: &ResolvedTarget, content: &str) -> CoreResult<FileMutationResult> {
        if target.canonical.as_path().exists() {
            return Err(CoreError::TargetExists(
                target.canonical.as_path().to_string_lossy().to_string(),
            ));
        }
        self.write_bytes(target, content.as_bytes())
    }

    /// Remove `target`, whether or not it exists.
    pub fn remove(&self, target: &ResolvedTarget) -> CoreResult<FileMutationResult> {
        let path = target.canonical.as_path();
        let existed = path.exists();
        if existed {
            std::fs::remove_file(path).map_err(|error| CoreError::FileSystem(error.to_string()))?;
        }
        Ok(FileMutationResult {
            operation: "remove".into(),
            target: target.canonical.clone(),
            resource: target.resource.clone(),
            existed,
        })
    }

    /// Write text, preserving exactly one BOM.
    pub fn write_text_preserving_bom(
        &self,
        target: &ResolvedTarget,
        content: &str,
    ) -> CoreResult<FileMutationResult> {
        let path = target.canonical.as_path();
        let current = std::fs::read(path).ok();
        let existed = current.is_some();
        let (content_bom, text) = split_bom(content);
        let current_bom = current.as_deref().map(has_utf8_bom).unwrap_or(false);
        let encoded = join_bom(text, current_bom || content_bom);
        write_with_dirs(path, encoded.as_bytes())?;
        Ok(FileMutationResult {
            operation: "write".into(),
            target: target.canonical.clone(),
            resource: target.resource.clone(),
            existed,
        })
    }

    /// Write only when the current bytes equal `expected`.
    pub fn write_if_unchanged(
        &self,
        target: &ResolvedTarget,
        expected: &[u8],
        content: &str,
    ) -> CoreResult<FileMutationResult> {
        let path = target.canonical.as_path();
        let current =
            std::fs::read(path).map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if current != expected {
            return Err(CoreError::StaleContent(path.to_string_lossy().to_string()));
        }
        self.write_bytes(target, content.as_bytes())
    }

    fn write_bytes(
        &self,
        target: &ResolvedTarget,
        content: &[u8],
    ) -> CoreResult<FileMutationResult> {
        let path = target.canonical.as_path();
        let existed = path.exists();
        write_with_dirs(path, content)?;
        Ok(FileMutationResult {
            operation: "write".into(),
            target: target.canonical.clone(),
            resource: target.resource.clone(),
            existed,
        })
    }
}

fn write_with_dirs(path: &Path, content: &[u8]) -> CoreResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        }
    }
    std::fs::write(path, content).map_err(|error| CoreError::FileSystem(error.to_string()))
}

fn split_bom(text: &str) -> (bool, &str) {
    let stripped = text.trim_start_matches('\u{feff}');
    (stripped.len() != text.len(), stripped)
}

fn join_bom(text: &str, bom: bool) -> String {
    let (_, stripped) = split_bom(text);
    if bom {
        format!("\u{feff}{stripped}")
    } else {
        stripped.to_string()
    }
}

fn has_utf8_bom(content: &[u8]) -> bool {
    content.len() >= 3 && content[0] == 0xef && content[1] == 0xbb && content[2] == 0xbf
}
