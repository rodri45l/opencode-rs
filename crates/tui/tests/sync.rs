//! Port of packages/tui/test/cli/cmd/tui/sync.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/cli/cmd/tui/sync.tsx; see docs/TEST-PORT.md.
//! The Solid runtime is re-derived as pure state.

use opencode_tui::sync::{session_refresh_query, should_apply_workspace_event, VcsState};

#[test]
fn refresh_scopes_sessions_by_default_and_lists_project_sessions_when_disabled() {
    let enabled = session_refresh_query(true, "packages/tui");
    assert_eq!(enabled.roots, None);
    assert_eq!(enabled.scope, None);
    assert_eq!(enabled.path.as_deref(), Some("packages/tui"));

    let disabled = session_refresh_query(false, "packages/tui");
    assert_eq!(disabled.scope.as_deref(), Some("project"));
    assert_eq!(disabled.path, None);
    assert_eq!(disabled.roots, None);
}

#[test]
fn vcs_branch_updates_only_apply_for_the_active_workspace() {
    let mut vcs = VcsState::new("main");
    assert_eq!(vcs.branch, "main");

    vcs.apply(Some("ws_a"), Some("ws_b"), "other");
    assert_eq!(vcs.branch, "main");

    vcs.apply(Some("ws_a"), Some("ws_a"), "feature");
    assert_eq!(vcs.branch, "feature");
}

#[test]
fn workspace_event_filter_matches_the_active_workspace() {
    assert!(!should_apply_workspace_event(Some("ws_a"), Some("ws_b")));
    assert!(should_apply_workspace_event(Some("ws_a"), Some("ws_a")));
}
