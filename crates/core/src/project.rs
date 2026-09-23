//! Project identity resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/project.ts`: a git
//! remote is normalized (ssh and https forms collapse to `host/owner/repo`,
//! `.git` suffix stripped) and used to derive a stable project id; `file://`
//! remotes are ignored; with no usable remote a root commit is used and with no
//! commit at all the project id is `global`. The live git repository discovery,
//! filesystem walk, and Effect `ProjectV2` service are dropped; the pure remote
//! normalization and id selection remain.

use crate::{CoreError, CoreResult};

/// The global project id.
pub const GLOBAL_ID: &str = "global";

/// Project identity helpers.
#[derive(Debug, Default)]
pub struct Project;

impl Project {
    /// Normalize a git remote to `host/path` (no scheme, no `.git`), or `None`
    /// for `file://` and local remotes.
    pub fn normalize_remote(_remote: &str) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "project::Project::normalize_remote",
        ))
    }

    /// The stable id key for a remote (`git-remote:<normalized>`), or `None`.
    pub fn remote_id_key(_remote: &str) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented("project::Project::remote_id_key"))
    }

    /// Resolve a project id key from an optional remote and optional root commit.
    /// A usable remote wins; otherwise the root commit; otherwise [`GLOBAL_ID`].
    pub fn resolve_id(_remote: Option<&str>, _root_commit: Option<&str>) -> CoreResult<String> {
        Err(CoreError::NotImplemented("project::Project::resolve_id"))
    }
}
