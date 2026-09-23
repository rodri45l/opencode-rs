//! Recently connected workspace selection.
//!
//! Port of packages/tui/src/component/dialog-workspace-create.tsx
//! `recentConnectedWorkspaces` behaviour (upstream 18ef3cc).

/// A workspace entry with its last-used timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceEntry {
    pub id: String,
    pub time_used: i64,
}

/// Connected workspaces ordered by recency, with a truncation flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentWorkspaces {
    pub recent: Vec<String>,
    pub has_more: bool,
}

/// Select the most recently used connected workspaces.
pub fn recent_connected_workspaces(
    workspaces: &[WorkspaceEntry],
    status: impl Fn(&str) -> Option<String>,
    limit: Option<usize>,
) -> RecentWorkspaces {
    let mut connected: Vec<&WorkspaceEntry> = workspaces
        .iter()
        .filter(|workspace| status(&workspace.id).as_deref() == Some("connected"))
        .collect();
    connected.sort_by_key(|workspace| std::cmp::Reverse(workspace.time_used));
    let take = limit.unwrap_or(3);
    let recent = connected
        .iter()
        .take(take)
        .map(|workspace| workspace.id.clone())
        .collect::<Vec<_>>();
    RecentWorkspaces {
        has_more: recent.len() < connected.len(),
        recent,
    }
}
