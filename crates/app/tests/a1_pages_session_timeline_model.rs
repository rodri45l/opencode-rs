//! Port of packages/app/src/pages/session/timeline/model.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    role: String,
}

// Local stubs (fast wave): real module lands later.
fn select_user_messages(_messages: &[Message]) -> Vec<Message> {
    Vec::new()
}

fn select_visible_user_messages(_users: &[Message], _revert: Option<&str>) -> Vec<Message> {
    Vec::new()
}

fn is_timeline_ready(_messages: &[Message], _more: bool) -> bool {
    false
}

#[derive(Clone, Debug, PartialEq)]
struct LoadOutcome {
    calls: usize,
    anchors: Vec<String>,
    restored: usize,
}

fn load_older_timeline(
    _session_id_before: &str,
    _session_id_after: &str,
    _more: bool,
    _loading: bool,
    _fails: bool,
) -> LoadOutcome {
    LoadOutcome {
        calls: 0,
        anchors: Vec::new(),
        restored: 0,
    }
}

fn user(id: &str) -> Message {
    Message {
        id: id.into(),
        role: "user".into(),
    }
}

fn assistant(id: &str) -> Message {
    Message {
        id: id.into(),
        role: "assistant".into(),
    }
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn selects_users_and_applies_the_revert_boundary() {
    let messages = vec![
        user("msg_z"),
        assistant("msg_a"),
        user("msg_b"),
        user("msg_c"),
    ];
    let users = select_user_messages(&messages);
    assert_eq!(
        users.iter().map(|m| m.id.clone()).collect::<Vec<_>>(),
        vec![
            "msg_z".to_string(),
            "msg_b".to_string(),
            "msg_c".to_string()
        ]
    );
    assert_eq!(
        select_visible_user_messages(&users, Some("msg_b"))
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_z".to_string()]
    );
    assert_eq!(select_visible_user_messages(&users, None), users);
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn waits_for_an_assistant_only_load_to_hydrate_its_user_root() {
    assert!(!is_timeline_ready(&[assistant("msg_2")], true));
    assert!(is_timeline_ready(
        &[user("msg_1"), assistant("msg_2")],
        true
    ));
    assert!(is_timeline_ready(&[], false));
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn loads_exactly_one_opaque_cursor_page() {
    let outcome = load_older_timeline("ses_test", "ses_test", true, false, false);
    assert_eq!(outcome.calls, 1);
    assert_eq!(
        outcome.anchors,
        vec![
            "before".to_string(),
            "after".to_string(),
            "true".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn stops_when_a_page_adds_no_raw_messages() {
    let outcome = load_older_timeline("ses_test", "ses_test", true, false, false);
    assert_eq!(outcome.calls, 1);
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn does_not_restore_an_anchor_after_the_session_changes() {
    let outcome = load_older_timeline("ses_old", "ses_new", true, false, false);
    assert_eq!(outcome.restored, 0);
}

#[test]
#[ignore = "porting: pages/session/timeline/model not implemented"]
fn releases_the_anchor_when_loading_history_fails() {
    let outcome = load_older_timeline("ses_test", "ses_test", true, false, true);
    assert_eq!(outcome.restored, 1);
}
