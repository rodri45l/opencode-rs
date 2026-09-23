//! Port of packages/app/src/context/global-sync/queue.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeSet;

#[derive(Default)]
struct RefreshQueue {
    queued: BTreeSet<String>,
    calls: Vec<String>,
}

impl RefreshQueue {
    // Local stubs (fast wave): real module lands later.
    fn push(&mut self, _directory: &str) {}
    fn clear(&mut self, _directory: &str) {}
    fn flush(&mut self) {}
    fn dispose(&mut self) {}
}

fn directory_key(path: &str) -> String {
    let replaced = if path.contains('\\') && !path.starts_with('/') {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };
    if replaced.len() > 1 {
        replaced.trim_end_matches('/').to_string()
    } else {
        replaced
    }
}

#[test]
#[ignore = "porting: context/global-sync/queue not implemented"]
fn clears_queued_directories_by_normalized_key() {
    let mut queue = RefreshQueue::default();
    queue.push("C:\\tmp\\demo");
    queue.clear("C:/tmp/demo");
    queue.flush();
    assert!(queue.calls.is_empty());
    queue.dispose();
}

#[test]
#[ignore = "porting: context/global-sync/queue not implemented"]
fn passes_the_original_directory_to_bootstrap_instance() {
    let mut queue = RefreshQueue::default();
    queue.push("C:\\tmp\\demo");
    queue.flush();
    assert_eq!(queue.calls, vec!["C:\\tmp\\demo".to_string()]);
    queue.dispose();
}

#[test]
#[ignore = "porting: context/global-sync/queue not implemented"]
fn directory_key_normalizes_slashes() {
    assert_eq!(directory_key("C:\\tmp\\demo"), "C:/tmp/demo");
}
