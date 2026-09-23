//! Layout page helpers.
//!
//! Port of `packages/app/src/pages/layout/helpers.ts` and
//! `packages/app/src/pages/layout/deep-links.ts` (upstream 18ef3cc).

use url::Url;

pub use crate::path_key::path_key;

/// A new-session deep link.
#[derive(Clone, Debug, PartialEq)]
pub struct NewSessionDeepLink {
    pub directory: String,
    pub prompt: Option<String>,
}

/// Parse an `opencode://open-project?directory=...` deep link.
pub fn parse_deep_link(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    if parsed.scheme() != "opencode" || parsed.host_str() != Some("open-project") {
        return None;
    }
    if !matches!(parsed.path(), "" | "/") {
        return None;
    }
    let directory = query_value(&parsed, "directory")?;
    if directory.is_empty() {
        return None;
    }
    Some(directory)
}

/// Collect every valid open-project directory from a list of deep links.
pub fn collect_open_project_deep_links(links: &[&str]) -> Vec<String> {
    links
        .iter()
        .filter_map(|link| parse_deep_link(link))
        .collect()
}

/// Parse an `opencode://new-session?directory=...&prompt=...` deep link.
pub fn parse_new_session_deep_link(url: &str) -> Option<NewSessionDeepLink> {
    let parsed = Url::parse(url).ok()?;
    if parsed.scheme() != "opencode" || parsed.host_str() != Some("new-session") {
        return None;
    }
    if !matches!(parsed.path(), "" | "/") {
        return None;
    }
    let directory = query_value(&parsed, "directory")?;
    if directory.is_empty() {
        return None;
    }
    let prompt = query_value(&parsed, "prompt").filter(|value| !value.is_empty());
    Some(NewSessionDeepLink { directory, prompt })
}

/// Collect every valid new-session deep link.
pub fn collect_new_session_deep_links(links: &[&str]) -> Vec<NewSessionDeepLink> {
    links
        .iter()
        .filter_map(|link| parse_new_session_deep_link(link))
        .collect()
}

/// Drain the pending global deep links.
pub fn drain_pending_deep_links(links: &mut Vec<String>) -> Vec<String> {
    std::mem::take(links)
}

fn query_value(url: &Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// Order workspace directories, keeping the local directory first.
pub fn effective_workspace_order(local: &str, known: &[&str], order: &[&str]) -> Vec<String> {
    let root = path_key(local);
    let mut live: Vec<(String, String)> = Vec::new();
    for dir in known {
        let key = path_key(dir);
        if key == root {
            continue;
        }
        if !live.iter().any(|(candidate, _)| *candidate == key) {
            live.push((key, (*dir).to_string()));
        }
    }

    if order.is_empty() {
        let mut result = vec![local.to_string()];
        result.extend(live.into_iter().map(|(_, value)| value));
        return result;
    }

    let mut result = vec![local.to_string()];
    for dir in order {
        let key = path_key(dir);
        if key == root {
            continue;
        }
        if let Some(index) = live.iter().position(|(candidate, _)| *candidate == key) {
            let (_, value) = live.remove(index);
            result.push(value);
        }
    }
    result.extend(live.into_iter().map(|(_, value)| value));
    result
}

/// A session row used by the layout helpers.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub directory: String,
    pub parent_id: Option<String>,
    pub archived: Option<i64>,
    pub created: i64,
    pub updated: i64,
}

/// Order sessions by update time descending, then id ascending.
pub fn compare_session_time(a: &Session, b: &Session) -> std::cmp::Ordering {
    b.updated.cmp(&a.updated).then_with(|| a.id.cmp(&b.id))
}

fn is_root_visible(session: &Session) -> bool {
    session.parent_id.is_none() && session.archived.is_none()
}

/// The most recently updated root session across all groups.
pub fn latest_root_session(groups: &[Vec<Session>], _window_ms: i64) -> Option<Session> {
    let mut roots: Vec<Session> = groups
        .iter()
        .flat_map(|group| group.iter())
        .filter(|session| is_root_visible(session))
        .cloned()
        .collect();
    roots.sort_by(compare_session_time);
    roots.into_iter().next()
}

/// Root sessions sorted by recency and truncated to `limit`.
pub fn sorted_root_sessions(sessions: Vec<Session>, limit: usize) -> Vec<Session> {
    let mut roots: Vec<Session> = sessions.into_iter().filter(is_root_visible).collect();
    roots.sort_by(compare_session_time);
    roots.truncate(limit);
    roots
}

/// Whether any permission in the map satisfies `include`.
pub fn has_project_permissions<T>(
    map: &std::collections::BTreeMap<String, Vec<T>>,
    include: impl Fn(&T) -> bool,
) -> bool {
    map.values().any(|list| list.iter().any(&include))
}

/// The direct child of `root` on the path to `target`.
pub fn child_session_on_path(list: &[Session], root: &str, target: &str) -> Option<Session> {
    if target.is_empty() || target == root {
        return None;
    }
    let mut id = target.to_string();
    loop {
        let session = list.iter().find(|session| session.id == id)?;
        match &session.parent_id {
            Some(parent) if parent == root => return Some(session.clone()),
            Some(parent) => id = parent.clone(),
            None => return None,
        }
    }
}

/// The display name of a project: explicit name, else the worktree basename.
pub fn display_name(worktree: &str, name: Option<&str>) -> String {
    if let Some(name) = name.filter(|name| !name.is_empty()) {
        return name.to_string();
    }
    let base = worktree
        .rsplit(['/', '\\'])
        .find(|segment| !segment.is_empty())
        .unwrap_or("");
    if base.is_empty() {
        worktree.to_string()
    } else {
        base.to_string()
    }
}

/// A home project selection.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectSelection {
    pub server: String,
    pub directory: Option<String>,
}

/// Toggle the selected directory for a server.
pub fn toggle_home_project_selection(
    current: Option<ProjectSelection>,
    server: &str,
    directory: &str,
) -> ProjectSelection {
    if let Some(current) = &current {
        if current.server == server && current.directory.as_deref() == Some(directory) {
            return ProjectSelection {
                server: server.to_string(),
                directory: None,
            };
        }
    }
    ProjectSelection {
        server: server.to_string(),
        directory: Some(directory.to_string()),
    }
}

/// Close a home project through its server context.
pub fn close_home_project(
    current: ProjectSelection,
    active_server: &str,
    closed: &mut Vec<String>,
    directory: &str,
) -> ProjectSelection {
    closed.push(directory.to_string());
    if current.server == active_server && current.directory.as_deref() == Some(directory) {
        return ProjectSelection {
            server: current.server,
            directory: None,
        };
    }
    current
}

/// A deferred home-project navigation.
#[derive(Clone, Debug, PartialEq)]
pub struct Navigation {
    pub server: Option<String>,
    pub href: String,
}

/// Defer navigation until the target server is active.
pub fn home_project_navigation(active: &str, server: &str, href: &str) -> Navigation {
    if active == server {
        Navigation {
            server: None,
            href: href.to_string(),
        }
    } else {
        Navigation {
            server: Some(server.to_string()),
            href: href.to_string(),
        }
    }
}

/// Normalize picker directories.
pub fn home_project_directories(input: Option<Vec<&str>>) -> Vec<String> {
    input
        .unwrap_or_default()
        .into_iter()
        .map(|directory| directory.to_string())
        .collect()
}

/// A server status projection.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerStatus {
    pub working: bool,
    pub tint: Option<String>,
}

/// Hide status derived from an inactive server.
pub fn home_session_server_status(active: bool, status: impl Fn() -> ServerStatus) -> ServerStatus {
    if !active {
        return ServerStatus {
            working: false,
            tint: None,
        };
    }
    status()
}

/// Extract an API error message, falling back when none is present.
pub fn error_message(error: &str, fallback: &str) -> String {
    if error.is_empty() || error == "unknown" {
        fallback.to_string()
    } else {
        error.to_string()
    }
}
