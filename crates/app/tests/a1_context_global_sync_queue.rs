//! Port of packages/app/src/context/global-sync/queue.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::global_sync_utils::directory_key;
use opencode_app::refresh_queue::RefreshQueue;

#[test]
fn clears_queued_directories_by_normalized_key() {
    let mut queue = RefreshQueue::default();
    queue.push("C:\\tmp\\demo");
    queue.clear("C:/tmp/demo");
    queue.flush();
    assert!(queue.calls.is_empty());
    queue.dispose();
}

#[test]
fn passes_the_original_directory_to_bootstrap_instance() {
    let mut queue = RefreshQueue::default();
    queue.push("C:\\tmp\\demo");
    queue.flush();
    assert_eq!(queue.calls, vec!["C:\\tmp\\demo".to_string()]);
    queue.dispose();
}

#[test]
fn directory_key_normalizes_slashes() {
    assert_eq!(directory_key("C:\\tmp\\demo"), "C:/tmp/demo");
}
