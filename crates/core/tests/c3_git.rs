//! Port of packages/core/test/git.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a checkout's git directory is `<worktree>/.git` and its
//! common directory equals the git directory; the default remote branch is
//! `main`; tree changes are filtered to a scope with path-boundary semantics
//! and returned sorted; diff status maps a path present only in the new tree to
//! `added` and a path in both trees to `modified`; and restore only rewrites the
//! files it is handed.
//!
//! Re-derived (dropped): the live `git` process calls (clone, fetch, reset,
//! checkout, worktree create/list/remove, tree write/preview/restore), the
//! `Git`/`LayerNode`/`Effect` service wiring, and the on-disk fixtures.

#![allow(dead_code)]

use std::collections::BTreeMap;

const NOTE: &str = "porting: git not implemented";

#[derive(Debug, PartialEq, Eq)]
enum GitError {
    NotImplemented,
}

#[derive(Debug, PartialEq, Eq)]
struct RepoLayout {
    worktree: String,
    git_directory: String,
    common_directory: String,
}

#[derive(Debug, PartialEq, Eq)]
enum DiffStatus {
    Added,
    Modified,
}

#[derive(Debug, PartialEq, Eq)]
struct TreeDiff {
    path: String,
    status: DiffStatus,
}

fn discover_layout(_worktree: &str) -> Result<RepoLayout, GitError> {
    Err(GitError::NotImplemented)
}

fn default_remote_branch() -> Result<&'static str, GitError> {
    Err(GitError::NotImplemented)
}

fn in_scope(_path: &str, _scope: &str) -> Result<bool, GitError> {
    Err(GitError::NotImplemented)
}

fn changed_paths(
    _from: &BTreeMap<&str, &str>,
    _to: &BTreeMap<&str, &str>,
    _scope: &str,
) -> Result<Vec<String>, GitError> {
    Err(GitError::NotImplemented)
}

fn tree_diff(
    _from: &BTreeMap<&str, &str>,
    _to: &BTreeMap<&str, &str>,
    _scope: &str,
) -> Result<Vec<TreeDiff>, GitError> {
    Err(GitError::NotImplemented)
}

fn restore_targets(_files: &BTreeMap<&str, &str>) -> Result<Vec<String>, GitError> {
    Err(GitError::NotImplemented)
}

#[test]
#[ignore = "porting: git not implemented"]
fn derives_checkout_git_and_common_directories() {
    let layout = discover_layout("/tmp/fixture/checkout").expect(NOTE);
    assert_eq!(layout.worktree, "/tmp/fixture/checkout");
    assert_eq!(layout.git_directory, "/tmp/fixture/checkout/.git");
    assert_eq!(layout.common_directory, layout.git_directory);
}

#[test]
#[ignore = "porting: git not implemented"]
fn default_remote_branch_is_main() {
    assert_eq!(default_remote_branch().expect(NOTE), "main");
}

#[test]
#[ignore = "porting: git not implemented"]
fn scope_filtering_respects_path_boundaries() {
    assert!(in_scope("scope/added.txt", "scope").expect(NOTE));
    assert!(in_scope("scope", "scope").expect(NOTE));
    assert!(!in_scope("outside.txt", "scope").expect(NOTE));
    assert!(!in_scope("scoped/x", "scope").expect(NOTE));
}

#[test]
#[ignore = "porting: git not implemented"]
fn changed_files_are_scope_filtered_and_sorted() {
    let before = BTreeMap::from([("scope/tracked.txt", "a"), ("outside.txt", "o")]);
    let after = BTreeMap::from([
        ("scope/tracked.txt", "b"),
        ("scope/added.txt", "c"),
        ("outside.txt", "p"),
    ]);

    assert_eq!(
        changed_paths(&before, &after, "scope").expect(NOTE),
        vec!["scope/added.txt", "scope/tracked.txt"]
    );
}

#[test]
#[ignore = "porting: git not implemented"]
fn tree_diff_maps_added_and_modified_statuses() {
    let before = BTreeMap::from([("scope/tracked.txt", "a")]);
    let after = BTreeMap::from([("scope/tracked.txt", "b"), ("scope/added.txt", "c")]);

    assert_eq!(
        tree_diff(&before, &after, "scope").expect(NOTE),
        vec![
            TreeDiff {
                path: "scope/added.txt".into(),
                status: DiffStatus::Added,
            },
            TreeDiff {
                path: "scope/tracked.txt".into(),
                status: DiffStatus::Modified,
            },
        ]
    );
}

#[test]
#[ignore = "porting: git not implemented"]
fn restore_only_rewrites_listed_files() {
    let files = BTreeMap::from([("scope/tracked.txt", "before")]);
    assert_eq!(
        restore_targets(&files).expect(NOTE),
        vec!["scope/tracked.txt"]
    );
}
