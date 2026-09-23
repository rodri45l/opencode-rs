//! Port of packages/app/src/context/global-sync/home-session-index.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

const HOME_V2_SESSION_PAGE_LIMIT: usize = 50;

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    parent_id: Option<String>,
    directory: String,
    title: String,
    created: i64,
    updated: i64,
    archived: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
struct SessionEvent {
    event_type: String,
    session_id: String,
    info: Option<Session>,
}

#[derive(Clone, Debug, PartialEq)]
struct IndexCache {
    sessions: Vec<Session>,
    event_sequence: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct RefreshDecision {
    connected: bool,
    refetch: bool,
}

// Local stubs (fast wave): real module lands later.
fn load_home_session_index(pages: &[Vec<Session>]) -> Vec<Session> {
    let _ = pages;
    Vec::new()
}

fn parse_home_session_index(_sessions: Vec<Session>) -> Vec<Session> {
    Vec::new()
}

fn retain_home_sessions(_sessions: Vec<Session>, _limit: usize, _now: i64) -> Vec<Session> {
    Vec::new()
}

fn apply_home_session_event(_sessions: Vec<Session>, _event: SessionEvent) -> Vec<Session> {
    Vec::new()
}

fn append_home_session_event(
    _events: Option<Vec<SessionEvent>>,
    _event: SessionEvent,
) -> Vec<SessionEvent> {
    Vec::new()
}

fn home_session_index_sessions(_index: &IndexCache, _events: &[SessionEvent]) -> Vec<Session> {
    Vec::new()
}

fn home_session_index_refresh(_event_type: &str, _mounted: bool) -> RefreshDecision {
    RefreshDecision {
        connected: false,
        refetch: false,
    }
}

fn remove_home_session(_index: Option<IndexCache>, _id: &str) -> Option<IndexCache> {
    None
}

fn session(
    id: &str,
    parent_id: Option<&str>,
    directory: &str,
    updated: i64,
    archived: Option<i64>,
) -> Session {
    Session {
        id: id.into(),
        parent_id: parent_id.map(str::to_string),
        directory: directory.into(),
        title: id.into(),
        created: 1,
        updated,
        archived,
    }
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn loads_the_home_index_with_one_global_v2_request() {
    let pages = vec![vec![session("root", None, "/project", 1, None)]];
    assert_eq!(load_home_session_index(&pages).len(), 1);
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn loads_subsequent_pages_until_the_session_index_is_complete() {
    let mut first = Vec::new();
    for index in 0..HOME_V2_SESSION_PAGE_LIMIT {
        first.push(session(
            &format!("page-1-{index}"),
            None,
            "/project",
            1,
            None,
        ));
    }
    let pages = vec![first, vec![session("page-2", None, "/project", 1, None)]];
    assert_eq!(
        load_home_session_index(&pages).len(),
        HOME_V2_SESSION_PAGE_LIMIT + 1
    );
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn maps_visible_roots_to_home_session_summaries() {
    let input = vec![
        session("root", None, "/project", 30, None),
        session("active-null", None, "/project", 20, None),
        session("child", Some("root"), "/project", 40, None),
        session("archived", None, "/project", 50, Some(50)),
    ];
    let result = parse_home_session_index(input);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "root");
    assert_eq!(result[0].directory, "/project");
    assert_eq!(result[1].id, "active-null");
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn preserves_the_per_directory_home_retention_limit() {
    let now = 10 * 60 * 60 * 1000;
    let mut sessions = Vec::new();
    for index in 0..80 {
        let directory = if index % 2 == 0 { "/one" } else { "/two" };
        sessions.push(session(
            &format!("session-{index}"),
            None,
            directory,
            index + 1,
            None,
        ));
    }
    let retained = retain_home_sessions(sessions, 10, now);
    assert_eq!(
        retained.iter().filter(|s| s.directory == "/one").count(),
        10
    );
    assert_eq!(
        retained.iter().filter(|s| s.directory == "/two").count(),
        10
    );
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn replays_session_events_over_the_loaded_index() {
    let initial = parse_home_session_index(vec![session("old", None, "/project", 1, None)]);
    let mut created = initial[0].clone();
    created.id = "new".into();
    created.title = "new".into();
    created.created = 2;
    created.updated = 2;

    let after_create = apply_home_session_event(
        initial,
        SessionEvent {
            event_type: "session.created".into(),
            session_id: created.id.clone(),
            info: Some(created.clone()),
        },
    );
    let after_delete = apply_home_session_event(
        after_create,
        SessionEvent {
            event_type: "session.deleted".into(),
            session_id: "old".into(),
            info: None,
        },
    );
    assert_eq!(after_delete, vec![created]);
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn applies_only_events_newer_than_the_index_baseline() {
    let initial = parse_home_session_index(vec![session("old", None, "/project", 1, None)]);
    let mut stale = initial[0].clone();
    stale.title = "stale".into();
    let mut current = initial[0].clone();
    current.title = "current".into();
    let first = append_home_session_event(
        None,
        SessionEvent {
            event_type: "session.updated".into(),
            session_id: stale.id.clone(),
            info: Some(stale),
        },
    );
    let events = append_home_session_event(
        Some(first),
        SessionEvent {
            event_type: "session.updated".into(),
            session_id: current.id.clone(),
            info: Some(current),
        },
    );
    let index = IndexCache {
        sessions: initial,
        event_sequence: 1,
    };
    assert_eq!(
        home_session_index_sessions(&index, &events)[0].title,
        "current".to_string()
    );
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn refetches_after_reconnect_disposal_and_session_moves() {
    assert_eq!(
        home_session_index_refresh("server.connected", false),
        RefreshDecision {
            connected: true,
            refetch: false
        }
    );
    assert_eq!(
        home_session_index_refresh("server.connected", true),
        RefreshDecision {
            connected: true,
            refetch: true
        }
    );
    assert!(home_session_index_refresh("global.disposed", true).refetch);
    assert!(home_session_index_refresh("session.next.moved", true).refetch);
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn removes_a_session_from_the_loaded_home_index() {
    let index = IndexCache {
        sessions: vec![
            session("a", None, "/project", 1, None),
            session("b", None, "/project", 1, None),
        ],
        event_sequence: 0,
    };
    let next = remove_home_session(Some(index), "a");
    assert_eq!(
        next.map(|i| i.sessions.iter().map(|s| s.id.clone()).collect::<Vec<_>>()),
        Some(vec!["b".to_string()])
    );
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn keeps_the_session_out_of_the_home_list_when_the_index_is_not_mounted() {
    let sessions = vec![
        session("a", None, "/project", 1, None),
        session("b", None, "/project", 1, None),
    ];
    let filtered = remove_home_session(None, "a");
    assert_eq!(filtered, None);
    let index = IndexCache {
        sessions,
        event_sequence: 0,
    };
    assert_eq!(
        home_session_index_sessions(&index, &[])
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec!["a".to_string(), "b".to_string()]
    );
}

#[test]
#[ignore = "porting: context/global-sync/home-session-index not implemented"]
fn page_limit_constant_is_fifty() {
    let _unused: BTreeMap<String, String> = BTreeMap::new();
    assert_eq!(HOME_V2_SESSION_PAGE_LIMIT, 50);
}
