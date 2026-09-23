//! Move a session between checkout directories.
//!
//! Ports the observable behaviour of
//! `packages/core/src/control-plane/move-session.ts`: moving a session to a
//! destination directory transfers in-scope changes (tracked, staged, and
//! untracked) when requested, and records the destination directory plus the
//! path relative to the destination worktree.

use std::path::{Path, PathBuf};

use crate::CoreResult;

/// A move destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveDestination {
    /// Destination directory.
    pub directory: String,
}

/// The result of moving a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveResult {
    /// Recorded session directory.
    pub directory: String,
    /// Recorded path relative to the destination worktree.
    pub path: String,
}

/// The move-session service.
#[derive(Debug, Default)]
pub struct MoveSession;

impl MoveSession {
    /// Move a session's changes from `source` to `destination`.
    pub fn move_session(
        _session_id: &str,
        source: &str,
        destination: &MoveDestination,
        move_changes: bool,
    ) -> CoreResult<MoveResult> {
        let source_path = Path::new(source);
        let destination_path = Path::new(&destination.directory);
        let path = destination_path
            .strip_prefix(source_path)
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if move_changes && !destination_path.starts_with(source_path) {
            copy_tree(source_path, destination_path)?;
        }
        Ok(MoveResult {
            directory: destination.directory.clone(),
            path,
        })
    }
}

fn copy_tree(source: &Path, destination: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(destination).map_err(file_error)?;
    let entries = std::fs::read_dir(source).map_err(file_error)?;
    for entry in entries {
        let entry = entry.map_err(file_error)?;
        let file_type = entry.file_type().map_err(file_error)?;
        let target: PathBuf = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), target).map_err(file_error)?;
        }
    }
    Ok(())
}

fn file_error(error: std::io::Error) -> crate::CoreError {
    crate::CoreError::FileSystem(error.to_string())
}
