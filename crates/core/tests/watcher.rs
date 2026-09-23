//! Port of packages/core/test/filesystem/watcher.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: only git roots publish watcher events, `.git/index` churn
//! is ignored, `.git/HEAD` changes are published, and ordinary file events map
//! to add/change/unlink. Re-derived: the native file-watcher binding, the
//! `FSUtil`/`Location`/`EventV2` service wiring, the experimental config flags,
//! and the live symlink/git fixtures are dropped.

use opencode_core::watcher::{Watcher, WatcherEventKind};

#[test]
fn only_git_roots_publish_events() {
    assert!(Watcher::should_publish(true, "watch.txt").unwrap());
    assert!(!Watcher::should_publish(false, "plain.txt").unwrap());
}

#[test]
fn ignores_git_index_but_publishes_git_head() {
    assert!(Watcher::is_git_index(".git/index").unwrap());
    assert!(!Watcher::should_publish(true, ".git/index").unwrap());
    assert!(Watcher::is_git_head(".git/HEAD").unwrap());
    assert!(Watcher::should_publish(true, ".git/HEAD").unwrap());
}

#[test]
fn maps_file_transitions_to_event_kinds() {
    assert_eq!(
        Watcher::event_kind(false, true).unwrap(),
        WatcherEventKind::Add
    );
    assert_eq!(
        Watcher::event_kind(true, true).unwrap(),
        WatcherEventKind::Change
    );
    assert_eq!(
        Watcher::event_kind(true, false).unwrap(),
        WatcherEventKind::Unlink
    );
}
