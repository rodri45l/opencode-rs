//! Port of packages/core/test/project.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: ssh and https git remotes normalize to the same
//! `host/owner/repo` identity, a remote wins over the root commit, a missing
//! remote falls back to the root commit, `file://` remotes are ignored, and with
//! neither remote nor commit the project id is `global`. Re-derived: the live
//! git repository discovery, `ProjectV2` service, worktree and cache `.git/opencode`
//! behaviour, and filesystem walk are dropped; the pure normalization and id
//! selection remain.

use opencode_core::project::{Project, GLOBAL_ID};

#[test]
fn normalizes_ssh_and_https_remotes_to_the_same_identity() {
    assert_eq!(
        Project::normalize_remote("git@github.com:owner/repo.git").unwrap(),
        Some("github.com/owner/repo".to_string())
    );
    assert_eq!(
        Project::normalize_remote("https://github.com/owner/repo.git").unwrap(),
        Some("github.com/owner/repo".to_string())
    );
    assert_eq!(
        Project::normalize_remote("git@github.com:Acme/App.git").unwrap(),
        Some("github.com/Acme/App".to_string())
    );
    assert_eq!(
        Project::remote_id_key("git@github.com:owner/repo.git").unwrap(),
        Project::remote_id_key("https://github.com/owner/repo.git").unwrap()
    );
}

#[test]
fn ignores_file_remotes_and_falls_back_to_the_root_commit() {
    assert!(Project::normalize_remote("file:///tmp/repo")
        .unwrap()
        .is_none());
    assert!(Project::remote_id_key("file:///tmp/repo")
        .unwrap()
        .is_none());
    assert_eq!(
        Project::resolve_id(Some("file:///tmp/repo"), Some("abc123")).unwrap(),
        "abc123"
    );
}

#[test]
fn prefers_a_normalized_remote_over_the_root_commit() {
    assert_eq!(
        Project::resolve_id(Some("https://github.com/owner/repo.git"), Some("abc123")).unwrap(),
        "git-remote:github.com/owner/repo"
    );
    assert_eq!(Project::resolve_id(None, Some("abc123")).unwrap(), "abc123");
    assert_eq!(Project::resolve_id(None, None).unwrap(), GLOBAL_ID);
}
