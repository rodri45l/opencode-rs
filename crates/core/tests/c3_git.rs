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
//! `Git`/`LayerNode`/`Effect` service wiring, and the on-disk fixtures. The pure
//! tree/scope semantics are exercised through [`opencode_core::git`].

#![allow(dead_code)]

use std::collections::BTreeMap;

use opencode_core::git::{
    changed_paths, default_remote_branch, discover_layout, in_scope, restore_targets, tree_diff,
    DiffStatus, TreeDiff,
};

#[test]
fn derives_checkout_git_and_common_directories() {
    let layout = discover_layout("/tmp/fixture/checkout").expect("layout");
    assert_eq!(layout.worktree, "/tmp/fixture/checkout");
    assert_eq!(layout.git_directory, "/tmp/fixture/checkout/.git");
    assert_eq!(layout.common_directory, layout.git_directory);
}

#[test]
fn default_remote_branch_is_main() {
    assert_eq!(default_remote_branch().expect("branch"), "main");
}

#[test]
fn scope_filtering_respects_path_boundaries() {
    assert!(in_scope("scope/added.txt", "scope").expect("scope"));
    assert!(in_scope("scope", "scope").expect("scope"));
    assert!(!in_scope("outside.txt", "scope").expect("scope"));
    assert!(!in_scope("scoped/x", "scope").expect("scope"));
}

#[test]
fn changed_files_are_scope_filtered_and_sorted() {
    let before = BTreeMap::from([("scope/tracked.txt", "a"), ("outside.txt", "o")]);
    let after = BTreeMap::from([
        ("scope/tracked.txt", "b"),
        ("scope/added.txt", "c"),
        ("outside.txt", "p"),
    ]);

    assert_eq!(
        changed_paths(&before, &after, "scope").expect("changed"),
        vec!["scope/added.txt", "scope/tracked.txt"]
    );
}

#[test]
fn tree_diff_maps_added_and_modified_statuses() {
    let before = BTreeMap::from([("scope/tracked.txt", "a")]);
    let after = BTreeMap::from([("scope/tracked.txt", "b"), ("scope/added.txt", "c")]);

    assert_eq!(
        tree_diff(&before, &after, "scope").expect("diff"),
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
fn restore_only_rewrites_listed_files() {
    let files = BTreeMap::from([("scope/tracked.txt", "before")]);
    assert_eq!(
        restore_targets(&files).expect("restore"),
        vec!["scope/tracked.txt"]
    );
}
