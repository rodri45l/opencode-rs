//! Port of packages/core/test/filesystem/watcher.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: only git roots publish watcher events, `.git/index` churn
//! is ignored, `.git/HEAD` changes are published, and ordinary file events map
//! to add/change/unlink. Re-derived: the native file-watcher binding, the
//! `FSUtil`/`Location`/`EventV2` service wiring, the experimental config flags,
//! and the live symlink/git fixtures are dropped.

use opencode_core::watcher::{Watcher, WatcherEventKind};

const NOTE: &str = "porting: filesystem watcher not implemented";

#[test]
#[ignore = "porting: filesystem watcher not implemented"]
fn only_git_roots_publish_events() {
    assert!(Watcher::should_publish(true, "watch.txt").expect(NOTE));
    assert!(!Watcher::should_publish(false, "plain.txt").expect(NOTE));
}

#[test]
#[ignore = "porting: filesystem watcher not implemented"]
fn ignores_git_index_but_publishes_git_head() {
    assert!(Watcher::is_git_index(".git/index").expect(NOTE));
    assert!(!Watcher::should_publish(true, ".git/index").expect(NOTE));
    assert!(Watcher::is_git_head(".git/HEAD").expect(NOTE));
    assert!(Watcher::should_publish(true, ".git/HEAD").expect(NOTE));
}

#[test]
#[ignore = "porting: filesystem watcher not implemented"]
fn maps_file_transitions_to_event_kinds() {
    assert_eq!(
        Watcher::event_kind(false, true).expect(NOTE),
        WatcherEventKind::Add
    );
    assert_eq!(
        Watcher::event_kind(true, true).expect(NOTE),
        WatcherEventKind::Change
    );
    assert_eq!(
        Watcher::event_kind(true, false).expect(NOTE),
        WatcherEventKind::Unlink
    );
}
