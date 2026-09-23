//! Project identity resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/project.ts`: a git
//! remote is normalized (ssh and https forms collapse to `host/owner/repo`,
//! `.git` suffix stripped) and used to derive a stable project id; `file://`
//! remotes are ignored; with no usable remote a root commit is used and with no
//! commit at all the project id is `global`. The live git repository discovery,
//! filesystem walk, and Effect `ProjectV2` service are dropped; the pure remote
//! normalization and id selection remain.

use crate::CoreResult;

/// The global project id.
pub const GLOBAL_ID: &str = "global";

/// Project identity helpers.
#[derive(Debug, Default)]
pub struct Project;

impl Project {
    /// Normalize a git remote to `host/path` (no scheme, no `.git`), or `None`
    /// for `file://` and local remotes.
    pub fn normalize_remote(remote: &str) -> CoreResult<Option<String>> {
        let trimmed = remote.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        if trimmed.starts_with("file://") {
            return Ok(None);
        }
        let identity = if let Some(rest) = trimmed.strip_prefix("git@") {
            match rest.split_once(':') {
                Some((host, path)) => format!("{host}/{}", path.trim_start_matches('/')),
                None => return Ok(None),
            }
        } else if let Some((_, rest)) = trimmed.split_once("://") {
            rest.to_string()
        } else if let Some((_, rest)) = trimmed.split_once('@') {
            match rest.split_once(':') {
                Some((host, path)) => format!("{host}/{}", path.trim_start_matches('/')),
                None => return Ok(None),
            }
        } else if trimmed.starts_with('/') {
            return Ok(None);
        } else {
            trimmed.to_string()
        };
        let identity = identity.strip_suffix(".git").unwrap_or(&identity);
        let identity = identity.trim_end_matches('/');
        if identity.is_empty() || !identity.contains('/') {
            return Ok(None);
        }
        Ok(Some(identity.to_string()))
    }

    /// The stable id key for a remote (`git-remote:<normalized>`), or `None`.
    pub fn remote_id_key(remote: &str) -> CoreResult<Option<String>> {
        Ok(Self::normalize_remote(remote)?.map(|normalized| format!("git-remote:{normalized}")))
    }

    /// Resolve a project id key from an optional remote and optional root commit.
    /// A usable remote wins; otherwise the root commit; otherwise [`GLOBAL_ID`].
    pub fn resolve_id(remote: Option<&str>, root_commit: Option<&str>) -> CoreResult<String> {
        if let Some(remote) = remote {
            if let Some(key) = Self::remote_id_key(remote)? {
                return Ok(key);
            }
        }
        if let Some(commit) = root_commit.filter(|commit| !commit.is_empty()) {
            return Ok(commit.to_string());
        }
        Ok(GLOBAL_ID.to_string())
    }
}
