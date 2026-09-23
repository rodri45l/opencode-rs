//! Port of packages/tui/test/component/dialog-session-list.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/component/dialog-session-list.tsx; see
//! docs/TEST-PORT.md. The pending-race case is re-expressed against the
//! synchronous loader (a response without data yields `None`).

use opencode_tui::dialog_session_list::{
    create_dialog_session_list_query, load_dialog_session_list, SessionListQuery,
};

#[test]
fn requests_root_sessions_for_the_default_browse_list() {
    assert_eq!(
        create_dialog_session_list_query(None, None, Some("packages/tui")),
        SessionListQuery {
            roots: true,
            limit: 100,
            search: None,
            scope: None,
            path: Some("packages/tui".to_string()),
        }
    );
}

#[test]
fn requests_root_sessions_for_search_results() {
    assert_eq!(
        create_dialog_session_list_query(Some(" deploy "), Some("project"), None),
        SessionListQuery {
            roots: true,
            limit: 30,
            search: Some("deploy".to_string()),
            scope: Some("project".to_string()),
            path: None,
        }
    );
}

#[test]
fn keeps_the_cache_usable_while_the_root_request_is_pending() {
    assert_eq!(
        load_dialog_session_list::<String, _>(None, None, None, |_query| Ok(None)),
        None
    );
}

#[test]
fn returns_the_loaded_list_on_success() {
    assert_eq!(
        load_dialog_session_list(None, None, None, |_query| Ok(Some(
            vec!["root".to_string()]
        ))),
        Some(vec!["root".to_string()])
    );
}

#[test]
fn falls_back_when_the_root_request_rejects() {
    assert_eq!(
        load_dialog_session_list::<String, _>(None, None, None, |_query| Err(())),
        None
    );
}
