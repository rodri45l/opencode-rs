//! Filesystem watcher event selection (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/filesystem/watcher.ts`:
//! only git roots are watched, `.git/index` churn is ignored, and `.git/HEAD`
//! changes are published alongside ordinary file add/change/unlink events. The
//! native file-watcher binding, the Effect `FSUtil`/`Location`/`EventV2` wiring,
//! and the experimental config flags are dropped; the pure publish decision
//! remains.

use crate::{CoreError, CoreResult};

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
    /// publish nothing; `.git/index` is always ignored.
    pub fn should_publish(_is_git_root: bool, _path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "watcher::Watcher::should_publish",
        ))
    }

    /// Whether a path is the git index.
    pub fn is_git_index(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("watcher::Watcher::is_git_index"))
    }

    /// Whether a path is the git HEAD.
    pub fn is_git_head(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("watcher::Watcher::is_git_head"))
    }

    /// The event kind for a change: `Add` when the file is new, `Unlink` when it
    /// is gone, otherwise `Change`.
    pub fn event_kind(_existed_before: bool, _exists_now: bool) -> CoreResult<WatcherEventKind> {
        Err(CoreError::NotImplemented("watcher::Watcher::event_kind"))
    }
}
