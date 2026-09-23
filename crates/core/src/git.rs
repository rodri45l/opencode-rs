//! Git repository helpers.
//!
//! Ports the observable, pure parts of `packages/core/src/git.ts`: checkout
//! directory derivation, the default remote branch, path-scope filtering with
//! boundary semantics, and the added/modified diff status of scoped trees. The
//! live `Git`/`LayerNode` service calls are not reproduced here.

use std::collections::{BTreeMap, BTreeSet};

/// Error raised by git helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    /// A generic git failure.
    Message(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::Message(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GitError {}

/// A checkout's worktree, git directory and common directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLayout {
    /// The worktree root.
    pub worktree: String,
    /// The per-worktree git directory.
    pub git_directory: String,
    /// The shared common directory.
    pub common_directory: String,
}

/// Diff status for a changed path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    /// The path is only present in the new tree.
    Added,
    /// The path is present in both trees with different content.
    Modified,
}

/// A changed path and its diff status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDiff {
    /// The path.
    pub path: String,
    /// The change status.
    pub status: DiffStatus,
}

/// Derive the worktree/git/common directories for a checkout.
pub fn discover_layout(worktree: &str) -> Result<RepoLayout, GitError> {
    let trimmed = worktree.trim_end_matches('/');
    let git_directory = format!("{trimmed}/.git");
    Ok(RepoLayout {
        worktree: worktree.to_string(),
        common_directory: git_directory.clone(),
        git_directory,
    })
}

/// The default remote branch name.
pub fn default_remote_branch() -> Result<&'static str, GitError> {
    Ok("main")
}

/// Whether `path` is inside `scope`, honouring path boundaries.
pub fn in_scope(path: &str, scope: &str) -> Result<bool, GitError> {
    let scope = scope.trim_end_matches('/');
    if path == scope {
        return Ok(true);
    }
    Ok(path
        .strip_prefix(scope)
        .map(|rest| rest.starts_with('/'))
        .unwrap_or(false))
}

/// Changed paths between two trees, filtered to `scope` and sorted.
pub fn changed_paths(
    from: &BTreeMap<&str, &str>,
    to: &BTreeMap<&str, &str>,
    scope: &str,
) -> Result<Vec<String>, GitError> {
    let mut paths = BTreeSet::new();
    for (path, value) in to {
        let previous = from.get(path);
        if previous != Some(value) && in_scope(path, scope)? {
            paths.insert((*path).to_string());
        }
    }
    for path in from.keys() {
        if !to.contains_key(path) && in_scope(path, scope)? {
            paths.insert((*path).to_string());
        }
    }
    Ok(paths.into_iter().collect())
}

/// Diff status for every changed path between two trees within `scope`.
pub fn tree_diff(
    from: &BTreeMap<&str, &str>,
    to: &BTreeMap<&str, &str>,
    scope: &str,
) -> Result<Vec<TreeDiff>, GitError> {
    let mut diffs = Vec::new();
    for path in changed_paths(from, to, scope)? {
        let status = if from.contains_key(path.as_str()) {
            DiffStatus::Modified
        } else {
            DiffStatus::Added
        };
        diffs.push(TreeDiff { path, status });
    }
    Ok(diffs)
}

/// The files a restore would rewrite, sorted.
pub fn restore_targets(files: &BTreeMap<&str, &str>) -> Result<Vec<String>, GitError> {
    Ok(files.keys().map(|path| (*path).to_string()).collect())
}
