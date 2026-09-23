//! Port of packages/app/src/pages/new-session/new-session-workspace-controller.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

// Local stubs (fast wave): real module lands later.
fn resolve_new_session_worktree(
    _enabled: bool,
    _selected: Option<&str>,
    _directory: &str,
    _project_worktree: &str,
) -> String {
    String::new()
}

fn normalize_new_session_worktree(
    _worktree: &str,
    _directory: &str,
    _project_worktree: &str,
) -> String {
    String::new()
}

fn resolve_new_session_branch(
    _worktree: &str,
    _local: &str,
    _worktree_branch: impl Fn(&str) -> Option<String>,
) -> String {
    String::new()
}

#[test]
#[ignore = "porting: pages/new-session-workspace-controller not implemented"]
fn uses_main_when_the_workspace_bar_is_unavailable() {
    assert_eq!(
        resolve_new_session_worktree(
            false,
            Some("/project/feature"),
            "/project/feature",
            "/project"
        ),
        "main"
    );
}

#[test]
#[ignore = "porting: pages/new-session-workspace-controller not implemented"]
fn derives_an_existing_worktree_from_the_current_directory() {
    assert_eq!(
        resolve_new_session_worktree(true, None, "/project/feature", "/project"),
        "/project/feature"
    );
    assert_eq!(
        resolve_new_session_worktree(true, None, "/project", "/project"),
        "main"
    );
}

#[test]
#[ignore = "porting: pages/new-session-workspace-controller not implemented"]
fn normalizes_main_to_the_project_root_outside_the_main_worktree() {
    assert_eq!(
        normalize_new_session_worktree("main", "/project/feature", "/project"),
        "/project"
    );
    assert_eq!(
        normalize_new_session_worktree("main", "/project", "/project"),
        "main"
    );
}

#[test]
#[ignore = "porting: pages/new-session-workspace-controller not implemented"]
fn falls_back_to_the_local_branch_for_main_create_and_unknown_worktrees() {
    let branch = |worktree: &str| {
        if worktree == "/project/feature" {
            Some("feature".to_string())
        } else {
            None
        }
    };
    assert_eq!(resolve_new_session_branch("main", "dev", branch), "dev");
    assert_eq!(resolve_new_session_branch("create", "dev", branch), "dev");
    assert_eq!(
        resolve_new_session_branch("/project/feature", "dev", branch),
        "feature"
    );
    assert_eq!(resolve_new_session_branch("/missing", "dev", branch), "dev");
}
