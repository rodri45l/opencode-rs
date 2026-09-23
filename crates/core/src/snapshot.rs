//! Location-scoped snapshots.
//!
//! Ports the observable behaviour of `packages/core/src/snapshot.ts`: capture
//! returns `None` outside a Git worktree, file lists between two snapshots are
//! relative and sorted, and restore/checkout revert tracked changes without
//! removing unrelated files.

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

impl Snapshot {
    /// Capture a snapshot of `directory`, or `None` outside Git.
    pub fn capture(_directory: &str) -> CoreResult<Option<SnapshotId>> {
        Err(CoreError::NotImplemented("snapshot::Snapshot::capture"))
    }

    /// Files changed between two snapshots, relative to the worktree.
    pub fn files(
        _directory: &str,
        _before: &SnapshotId,
        _after: &SnapshotId,
    ) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented("snapshot::Snapshot::files"))
    }

    /// A preview of a file at a snapshot.
    pub fn preview(_directory: &str, _file: &str, _snapshot: &SnapshotId) -> CoreResult<String> {
        Err(CoreError::NotImplemented("snapshot::Snapshot::preview"))
    }

    /// Restore the listed files to their snapshots.
    pub fn restore(_directory: &str, _files: &[(String, SnapshotId)]) -> CoreResult<()> {
        Err(CoreError::NotImplemented("snapshot::Snapshot::restore"))
    }

    /// Check out a snapshot without removing unrelated files.
    pub fn checkout(_directory: &str, _snapshot: &SnapshotId) -> CoreResult<()> {
        Err(CoreError::NotImplemented("snapshot::Snapshot::checkout"))
    }
}
