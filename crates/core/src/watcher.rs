//! Filesystem watcher event selection (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/filesystem/watcher.ts`:
//! only git roots are watched, `.git/index` churn is ignored, and `.git/HEAD`
//! changes are published alongside ordinary file add/change/unlink events. The
//! native file-watcher binding, the Effect `FSUtil`/`Location`/`EventV2` wiring,
//! and the experimental config flags are dropped; the pure publish decision
//! remains.

use crate::CoreResult;

/// A published watcher event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatcherEventKind {
    /// A file was created.
    Add,
    /// A file changed.
    Change,
    /// A file was removed.
    Unlink,
}

/// The filesystem watcher selection helpers.
#[derive(Debug, Default)]
pub struct Watcher;

impl Watcher {
    /// Whether an event for `path` in a root should be published. Non-git roots
    /// publish nothing; `.git/index` churn is ignored.
    pub fn should_publish(is_git_root: bool, path: &str) -> CoreResult<bool> {
        if !is_git_root {
            return Ok(false);
        }
        if Self::is_git_index(path)? {
            return Ok(false);
        }
        Ok(true)
    }

    /// Whether `path` is the git index file.
    pub fn is_git_index(path: &str) -> CoreResult<bool> {
        let normalized = path.replace('\\', "/");
        Ok(normalized == ".git/index" || normalized.ends_with("/.git/index"))
    }

    /// Whether `path` is the git HEAD file.
    pub fn is_git_head(path: &str) -> CoreResult<bool> {
        let normalized = path.replace('\\', "/");
        Ok(normalized == ".git/HEAD" || normalized.ends_with("/.git/HEAD"))
    }

    /// Map `(existed_before, exists_after)` to an event kind.
    pub fn event_kind(existed_before: bool, exists_after: bool) -> CoreResult<WatcherEventKind> {
        Ok(match (existed_before, exists_after) {
            (false, true) => WatcherEventKind::Add,
            (true, true) => WatcherEventKind::Change,
            (true, false) => WatcherEventKind::Unlink,
            (false, false) => WatcherEventKind::Change,
        })
    }
}
