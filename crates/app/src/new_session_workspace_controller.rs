//! New session workspace controller (port of
//! packages/app/src/pages/new-session/new-session-workspace-controller.ts).

pub fn resolve_new_session_worktree(
    enabled: bool,
    selected: Option<&str>,
    directory: &str,
    project_worktree: &str,
) -> String {
    if !enabled {
        return "main".to_string();
    }
    if let Some(selected) = selected {
        return selected.to_string();
    }
    if !project_worktree.is_empty() && directory != project_worktree {
        return directory.to_string();
    }
    "main".to_string()
}

pub fn normalize_new_session_worktree(
    worktree: &str,
    directory: &str,
    project_worktree: &str,
) -> String {
    if worktree == "main" && project_worktree != directory {
        return project_worktree.to_string();
    }
    worktree.to_string()
}

pub fn resolve_new_session_branch(
    worktree: &str,
    local: &str,
    worktree_branch: impl Fn(&str) -> Option<String>,
) -> String {
    if worktree == "main" || worktree == "create" {
        return local.to_string();
    }
    worktree_branch(worktree).unwrap_or_else(|| local.to_string())
}
