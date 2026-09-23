//! Move a session between checkout directories.
//!
//! Ports the observable behaviour of
//! `packages/core/src/control-plane/move-session.ts`: moving a session to a
//! destination directory transfers in-scope changes (tracked, staged, and
//! untracked) when requested, and records the destination directory plus the
//! path relative to the destination worktree.

use crate::{CoreError, CoreResult};

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
        _source: &str,
        _destination: &MoveDestination,
        _move_changes: bool,
    ) -> CoreResult<MoveResult> {
        Err(CoreError::NotImplemented(
            "move_session::MoveSession::move_session",
        ))
    }
}
