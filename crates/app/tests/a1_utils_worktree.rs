//! Port of packages/app/src/utils/worktree.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::worktree::{Worktree, WorktreeState};

fn normalize(key: &str) -> String {
    key.trim_end_matches('/').to_string()
}

#[test]
fn normalizes_trailing_slashes() {
    let mut worktree = Worktree::default();
    let key = "/tmp/opencode-worktree-normalize";
    worktree.ready("local", &normalize(&format!("{key}/")));
    assert_eq!(worktree.get("local", key), Some(WorktreeState::Ready));
}

#[test]
fn pending_does_not_overwrite_a_terminal_state() {
    let mut worktree = Worktree::default();
    let key = "/tmp/opencode-worktree-pending";
    worktree.failed("local", key, "boom");
    worktree.pending("local", key);
    assert_eq!(
        worktree.get("local", key),
        Some(WorktreeState::Failed("boom".to_string()))
    );
}

#[test]
fn wait_resolves_shared_pending_waiter_when_ready() {
    let mut worktree = Worktree::default();
    let key = "/tmp/opencode-worktree-wait-ready";
    worktree.pending("local", key);
    worktree.ready("local", key);
    assert_eq!(worktree.get("local", key), Some(WorktreeState::Ready));
    assert_eq!(
        worktree.get("local", &format!("{key}/")),
        Some(WorktreeState::Ready)
    );
}

#[test]
fn wait_resolves_with_failure_message() {
    let mut worktree = Worktree::default();
    let key = "/tmp/opencode-worktree-wait-failed";
    worktree.failed("local", key, "permission denied");
    assert_eq!(
        worktree.get("local", key),
        Some(WorktreeState::Failed("permission denied".to_string()))
    );
    assert_eq!(
        worktree.get("local", key),
        Some(WorktreeState::Failed("permission denied".to_string()))
    );
}

#[test]
fn isolates_identical_directories_by_server_scope() {
    let mut worktree = Worktree::default();
    let key = "/tmp/opencode-worktree-scope";
    worktree.ready("local", key);
    worktree.failed("https://debian.example", key, "remote failed");
    assert_eq!(worktree.get("local", key), Some(WorktreeState::Ready));
    assert_eq!(
        worktree.get("https://debian.example", key),
        Some(WorktreeState::Failed("remote failed".to_string()))
    );
}
