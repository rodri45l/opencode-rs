//! Session list query construction and loading.
//!
//! Port of packages/tui/src/component/dialog-session-list.tsx
//! `createDialogSessionListQuery`/`loadDialogSessionList` behaviour (upstream
//! 18ef3cc). The pending-race case is re-expressed against the synchronous
//! loader (a response without data yields `None`).

/// A session-list query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionListQuery {
    pub roots: bool,
    pub limit: u32,
    pub search: Option<String>,
    pub scope: Option<String>,
    pub path: Option<String>,
}

/// Build the session-list query for a browse or search request.
pub fn create_dialog_session_list_query(
    search: Option<&str>,
    scope: Option<&str>,
    path: Option<&str>,
) -> SessionListQuery {
    let search = search.map(str::trim).filter(|value| !value.is_empty());
    SessionListQuery {
        roots: true,
        limit: if search.is_some() { 30 } else { 100 },
        search: search.map(str::to_string),
        scope: scope.map(str::to_string),
        path: path.map(str::to_string),
    }
}

/// Load a session list, returning `None` for missing data or errors.
pub fn load_dialog_session_list<T, F>(
    search: Option<&str>,
    scope: Option<&str>,
    path: Option<&str>,
    list: F,
) -> Option<Vec<T>>
where
    F: FnOnce(&SessionListQuery) -> Result<Option<Vec<T>>, ()>,
{
    let query = create_dialog_session_list_query(search, scope, path);
    list(&query).ok().flatten()
}
