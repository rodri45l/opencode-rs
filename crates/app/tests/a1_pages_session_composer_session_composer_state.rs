//! Port of packages/app/src/pages/session/composer/session-composer-state.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    parent_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Request {
    id: String,
    session_id: String,
}

// Local stubs (fast wave): real module lands later.
fn session_permission_request(
    _sessions: &[Session],
    _requests: &BTreeMap<String, Vec<Request>>,
    _root: &str,
    _filter: impl Fn(&Request) -> bool,
) -> Option<Request> {
    None
}

fn session_question_request(
    _sessions: &[Session],
    _requests: &BTreeMap<String, Vec<Request>>,
    _root: &str,
) -> Option<Request> {
    None
}

fn todo_state(_count: usize, _done: bool, _live: bool) -> String {
    String::new()
}

fn todo_dock_at_boundary(_state: &str) -> bool {
    false
}

fn session(id: &str, parent_id: Option<&str>) -> Session {
    Session {
        id: id.into(),
        parent_id: parent_id.map(str::to_string),
    }
}

fn request(id: &str, session_id: &str) -> Request {
    Request {
        id: id.into(),
        session_id: session_id.into(),
    }
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn prefers_the_current_session_permission() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut requests = BTreeMap::new();
    requests.insert("root".to_string(), vec![request("perm-root", "root")]);
    requests.insert("child".to_string(), vec![request("perm-child", "child")]);
    assert_eq!(
        session_permission_request(&sessions, &requests, "root", |_| true).map(|r| r.id),
        Some("perm-root".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn returns_a_nested_child_permission() {
    let sessions = vec![
        session("root", None),
        session("child", Some("root")),
        session("grand", Some("child")),
        session("other", None),
    ];
    let mut requests = BTreeMap::new();
    requests.insert("grand".to_string(), vec![request("perm-grand", "grand")]);
    requests.insert("other".to_string(), vec![request("perm-other", "other")]);
    assert_eq!(
        session_permission_request(&sessions, &requests, "root", |_| true).map(|r| r.id),
        Some("perm-grand".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn returns_undefined_without_a_matching_tree_permission() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut requests = BTreeMap::new();
    requests.insert("other".to_string(), vec![request("perm-other", "other")]);
    assert_eq!(
        session_permission_request(&sessions, &requests, "root", |_| true),
        None
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn skips_filtered_permissions_in_the_current_tree() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut requests = BTreeMap::new();
    requests.insert("root".to_string(), vec![request("perm-root", "root")]);
    requests.insert("child".to_string(), vec![request("perm-child", "child")]);
    assert_eq!(
        session_permission_request(&sessions, &requests, "root", |item| item.id != "perm-root")
            .map(|r| r.id),
        Some("perm-child".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn returns_undefined_when_all_tree_permissions_are_filtered_out() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut requests = BTreeMap::new();
    requests.insert("root".to_string(), vec![request("perm-root", "root")]);
    requests.insert("child".to_string(), vec![request("perm-child", "child")]);
    assert_eq!(
        session_permission_request(&sessions, &requests, "root", |_| false),
        None
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn prefers_the_current_session_question() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut requests = BTreeMap::new();
    requests.insert("root".to_string(), vec![request("q-root", "root")]);
    requests.insert("child".to_string(), vec![request("q-child", "child")]);
    assert_eq!(
        session_question_request(&sessions, &requests, "root").map(|r| r.id),
        Some("q-root".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn returns_a_nested_child_question() {
    let sessions = vec![
        session("root", None),
        session("child", Some("root")),
        session("grand", Some("child")),
    ];
    let mut requests = BTreeMap::new();
    requests.insert("grand".to_string(), vec![request("q-grand", "grand")]);
    assert_eq!(
        session_question_request(&sessions, &requests, "root").map(|r| r.id),
        Some("q-grand".to_string())
    );
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn hides_when_there_are_no_todos() {
    assert_eq!(todo_state(0, false, true), "hide");
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn opens_while_the_session_is_still_working() {
    assert_eq!(todo_state(2, false, true), "open");
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn closes_completed_todos_after_a_running_turn() {
    assert_eq!(todo_state(2, true, true), "close");
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn clears_stale_todos_when_the_turn_ends() {
    assert_eq!(todo_state(2, false, false), "clear");
    assert_eq!(todo_state(2, true, false), "clear");
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn shows_active_todos_when_entering_a_session() {
    assert!(todo_dock_at_boundary("open"));
}

#[test]
#[ignore = "porting: pages/session/composer/session-composer-state not implemented"]
fn hides_completed_todos_when_entering_a_session() {
    assert!(!todo_dock_at_boundary("close"));
}
