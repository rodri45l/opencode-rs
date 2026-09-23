//! Session sync helpers.
//!
//! Port of packages/tui/src/cli/cmd/tui/sync.test.tsx behaviour (upstream
//! 18ef3cc): directory-scoped refresh queries and workspace-scoped VCS events.
//! The Solid runtime is re-derived as pure state.

/// The query parameters used when refreshing the session list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionRefreshQuery {
    pub roots: Option<bool>,
    pub scope: Option<String>,
    pub path: Option<String>,
}

/// Build the session refresh query for the current directory-filter setting.
pub fn session_refresh_query(filter_enabled: bool, path: &str) -> SessionRefreshQuery {
    if filter_enabled {
        SessionRefreshQuery {
            roots: None,
            scope: None,
            path: Some(path.to_string()),
        }
    } else {
        SessionRefreshQuery {
            roots: None,
            scope: Some("project".to_string()),
            path: None,
        }
    }
}

/// Whether a workspace-scoped event applies to the active workspace.
pub fn should_apply_workspace_event(active: Option<&str>, event: Option<&str>) -> bool {
    active == event
}

/// The VCS branch state for the active workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcsState {
    pub branch: String,
    pub workspace: Option<String>,
}

impl VcsState {
    /// Create VCS state with the given initial branch.
    pub fn new(branch: impl Into<String>) -> Self {
        VcsState {
            branch: branch.into(),
            workspace: None,
        }
    }

    /// Apply a branch event if it targets the active workspace.
    pub fn apply(
        &mut self,
        active_workspace: Option<&str>,
        event_workspace: Option<&str>,
        branch: &str,
    ) {
        if should_apply_workspace_event(active_workspace, event_workspace) {
            self.branch = branch.to_string();
        }
    }
}
