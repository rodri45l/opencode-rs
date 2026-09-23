//! Port of packages/app/src/context/permission-auto-respond.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

use opencode_test_support as ts;

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    parent_id: Option<String>,
}

type AutoAccept = BTreeMap<String, bool>;

// Local stubs (fast wave): real module lands later.
fn auto_responds_permission(
    _auto_accept: &AutoAccept,
    _sessions: &[Session],
    _session_id: &str,
    _directory: &str,
) -> bool {
    false
}

fn session_auto_accept(
    _auto_accept: &AutoAccept,
    _sessions: &[Session],
    _session_id: &str,
    _directory: &str,
) -> Option<bool> {
    None
}

fn is_directory_auto_accepting(_auto_accept: &AutoAccept, _directory: &str) -> bool {
    false
}

fn key(directory: &str, session: &str) -> String {
    format!("{}/{}", ts::encode_b64url(directory.as_bytes()), session)
}

fn dir_key(directory: &str) -> String {
    format!("{}/*", ts::encode_b64url(directory.as_bytes()))
}

fn session(id: &str, parent_id: Option<&str>) -> Session {
    Session {
        id: id.into(),
        parent_id: parent_id.map(str::to_string),
    }
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn uses_a_parent_sessions_directory_scoped_auto_accept() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(key(directory, "root"), true);
    assert!(auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn uses_a_parent_sessions_legacy_auto_accept_key() {
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert("root".into(), true);
    assert!(auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        "/tmp/project"
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn defaults_to_requiring_approval_when_no_lineage_override_exists() {
    let sessions = vec![
        session("root", None),
        session("child", Some("root")),
        session("other", None),
    ];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert("other".into(), true);
    assert!(!auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        "/tmp/project"
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn inherits_a_parent_sessions_false_override() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(key(directory, "root"), false);
    assert!(!auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn prefers_a_child_override_over_parent_override() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(key(directory, "root"), false);
    auto_accept.insert(key(directory, "child"), true);
    assert!(auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn falls_back_to_directory_level_auto_accept() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None)];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), true);
    assert!(auto_responds_permission(
        &auto_accept,
        &sessions,
        "root",
        directory
    ));
    assert_eq!(
        session_auto_accept(&auto_accept, &sessions, "root", directory),
        None
    );
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn session_level_override_takes_precedence_over_directory_level() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None)];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), true);
    auto_accept.insert(key(directory, "root"), false);
    assert!(!auto_responds_permission(
        &auto_accept,
        &sessions,
        "root",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn parent_false_override_takes_precedence_over_directory_level_auto_accept() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), true);
    auto_accept.insert(key(directory, "root"), false);
    assert!(!auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn parent_true_override_takes_precedence_over_disabled_directory_fallback() {
    let directory = "/tmp/project";
    let sessions = vec![session("root", None), session("child", Some("root"))];
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), false);
    auto_accept.insert(key(directory, "root"), true);
    assert!(auto_responds_permission(
        &auto_accept,
        &sessions,
        "child",
        directory
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn is_directory_auto_accepting_returns_true_when_directory_key_is_set() {
    let directory = "/tmp/project";
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), true);
    assert!(is_directory_auto_accepting(&auto_accept, directory));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn is_directory_auto_accepting_returns_false_when_directory_key_is_not_set() {
    assert!(!is_directory_auto_accepting(
        &AutoAccept::new(),
        "/tmp/project"
    ));
}

#[test]
#[ignore = "porting: context/permission-auto-respond not implemented"]
fn is_directory_auto_accepting_returns_false_when_directory_key_is_explicitly_false() {
    let directory = "/tmp/project";
    let mut auto_accept = AutoAccept::new();
    auto_accept.insert(dir_key(directory), false);
    assert!(!is_directory_auto_accepting(&auto_accept, directory));
}
