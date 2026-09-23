//! Location-scoped snapshots.
//!
//! Ports the observable behaviour of `packages/core/src/snapshot.ts`: capture
//! returns `None` outside a Git worktree, file lists between two snapshots are
//! relative and sorted, and restore/checkout revert tracked changes without
//! removing unrelated files. Snapshots are Git tree objects written through a
//! private index so the working tree and normal index are left untouched.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{CoreError, CoreResult};

/// An opaque snapshot reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SnapshotId {
    /// Snapshot identifier.
    pub id: String,
}

/// Snapshot service helpers.
#[derive(Debug, Default)]
pub struct Snapshot;

static COUNTER: AtomicU64 = AtomicU64::new(0);

impl Snapshot {
    /// Capture a snapshot of `directory`, or `None` outside Git.
    pub fn capture(directory: &str) -> CoreResult<Option<SnapshotId>> {
        if !Self::is_git(directory) {
            return Ok(None);
        }
        let index = std::env::temp_dir().join(format!(
            "opencode-rs-snapshot-index-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&index);

        let add = Command::new("git")
            .args(["add", "-A"])
            .current_dir(directory)
            .env("GIT_INDEX_FILE", &index)
            .output()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if !add.status.success() {
            let _ = std::fs::remove_file(&index);
            return Err(CoreError::Message(
                String::from_utf8_lossy(&add.stderr).trim().to_string(),
            ));
        }
        let tree = Command::new("git")
            .args(["write-tree"])
            .current_dir(directory)
            .env("GIT_INDEX_FILE", &index)
            .output()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        let _ = std::fs::remove_file(&index);
        if !tree.status.success() {
            return Err(CoreError::Message(
                String::from_utf8_lossy(&tree.stderr).trim().to_string(),
            ));
        }
        let hash = String::from_utf8_lossy(&tree.stdout).trim().to_string();
        if hash.is_empty() {
            return Ok(None);
        }
        Ok(Some(SnapshotId { id: hash }))
    }

    /// Files changed between two snapshots, relative to the worktree.
    pub fn files(
        directory: &str,
        before: &SnapshotId,
        after: &SnapshotId,
    ) -> CoreResult<Vec<String>> {
        let output = Command::new("git")
            .args(["diff", "--name-only", &before.id, &after.id])
            .current_dir(directory)
            .output()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::Message(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        let mut files: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        files.sort();
        Ok(files)
    }

    /// A preview of a file at a snapshot.
    pub fn preview(directory: &str, file: &str, snapshot: &SnapshotId) -> CoreResult<String> {
        let output = Command::new("git")
            .args(["show", &format!("{}:{file}", snapshot.id)])
            .current_dir(directory)
            .output()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::Message(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Restore the listed files to their snapshots.
    pub fn restore(directory: &str, files: &[(String, SnapshotId)]) -> CoreResult<()> {
        for (file, snapshot) in files {
            let output = Command::new("git")
                .args(["checkout", &snapshot.id, "--", file])
                .current_dir(directory)
                .output()
                .map_err(|error| CoreError::FileSystem(error.to_string()))?;
            if !output.status.success() {
                return Err(CoreError::Message(
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Check out a snapshot without removing unrelated files.
    pub fn checkout(directory: &str, snapshot: &SnapshotId) -> CoreResult<()> {
        let output = Command::new("git")
            .args(["checkout", &snapshot.id, "--", "."])
            .current_dir(directory)
            .output()
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        if !output.status.success() {
            return Err(CoreError::Message(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        Ok(())
    }

    fn is_git(directory: &str) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(Path::new(directory))
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// The absolute directory a snapshot belongs to.
    pub fn directory_of(path: &str) -> PathBuf {
        Path::new(path).to_path_buf()
    }
}
