//! Port of packages/core/test/project-copy.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: strategy ids must be non-empty and carry no surrounding
//! whitespace; re-registering a strategy is a duplicate error and an unknown id
//! reports itself as unavailable; a copy directory gets the first free numeric
//! suffix (`copy`, `copy-2`, ...) and fails after ten conflicts at `copy-10`;
//! and refresh reports only added/removed directories, removing missing
//! ordinary checkouts and treating a missing project as a no-op.
//!
//! Re-derived (dropped): the `ProjectCopy`/`Database`/`EventV2` service wiring,
//! the live git worktree create/remove, the `ProjectDirectoryTable` rows, and
//! the event-stream assertions.

#![allow(dead_code)]

use opencode_core::project_copy::{
    candidate_directory_name, parse_strategy_id, refresh_diff, register_strategy, require_strategy,
    resolve_directory, CopyError,
};

#[test]
fn accepts_arbitrary_non_empty_strategy_ids() {
    assert_eq!(
        parse_strategy_id("acme/snapshot").expect("id").0,
        "acme/snapshot"
    );
    assert_eq!(
        parse_strategy_id("  acme/snapshot  "),
        Err(CopyError::InvalidStrategyId)
    );
    assert_eq!(parse_strategy_id("   "), Err(CopyError::InvalidStrategyId));
}

#[test]
fn rejects_duplicate_strategies_and_reports_unavailable_ids() {
    let known = vec!["test/duplicate".to_string()];

    assert_eq!(
        register_strategy(&known, "test/duplicate"),
        Err(CopyError::DuplicateStrategy)
    );
    assert_eq!(register_strategy(&known, "acme/new"), Ok(()));
    assert_eq!(
        require_strategy(&known, "acme/missing"),
        Err(CopyError::StrategyUnavailable("acme/missing".into()))
    );
}

#[test]
fn numbers_copy_directories_after_the_first() {
    assert_eq!(candidate_directory_name("copy", 1).expect("name"), "copy");
    assert_eq!(candidate_directory_name("copy", 3).expect("name"), "copy-3");
}

#[test]
fn adds_a_numeric_suffix_when_the_copy_directory_exists() {
    let existing = ["/p/copy", "/p/copy-2"];
    let resolved =
        resolve_directory("/p", "copy", |path| existing.contains(&path)).expect("resolve");
    assert_eq!(resolved, "/p/copy-3");
}

#[test]
fn fails_after_ten_copy_directory_conflicts() {
    let existing = [
        "/p/copy",
        "/p/copy-2",
        "/p/copy-3",
        "/p/copy-4",
        "/p/copy-5",
        "/p/copy-6",
        "/p/copy-7",
        "/p/copy-8",
        "/p/copy-9",
        "/p/copy-10",
    ];

    assert_eq!(
        resolve_directory("/p", "copy", |path| existing.contains(&path)),
        Err(CopyError::DestinationExists("/p/copy-10".into()))
    );
}

#[test]
fn refresh_reports_removed_missing_checkouts() {
    let diff = refresh_diff(&["/root", "/root-missing"], &["/root"]).expect("diff");
    assert_eq!(diff.updated, Vec::<String>::new());
    assert_eq!(diff.removed, vec!["/root-missing"]);
}

#[test]
fn refresh_with_no_roots_is_a_noop() {
    let diff = refresh_diff(&[], &[]).expect("diff");
    assert!(diff.updated.is_empty());
    assert!(diff.removed.is_empty());
}

#[test]
fn refresh_without_changes_reports_nothing() {
    let diff = refresh_diff(&["/root"], &["/root"]).expect("diff");
    assert!(diff.updated.is_empty());
    assert!(diff.removed.is_empty());
}
