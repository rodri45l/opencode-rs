//! Port of packages/app/src/utils/session-route.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::HashMap;

use opencode_test_support as ts;

#[derive(Clone, Debug, PartialEq)]
struct Tab {
    kind: String,
    server: String,
    session_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    parent_id: Option<String>,
}

// Local stubs (fast wave): real module lands later.
fn legacy_session_server(_tabs: &[Tab], _session_id: &str, _active: &str) -> String {
    String::new()
}

fn session_href(_server: &str, _session_id: &str) -> String {
    String::new()
}

fn require_server_key(_key: &str) -> Result<String, String> {
    Err("Invalid server route".to_string())
}

fn legacy_session_href(_directory: &str, _session_id: &str) -> String {
    String::new()
}

fn root_session(session: Session, sessions: &HashMap<String, Session>) -> Result<Session, String> {
    let _ = session;
    let _ = sessions;
    Err("not implemented".to_string())
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn uses_the_unique_persisted_server_for_a_legacy_session_route() {
    let tabs = vec![Tab {
        kind: "session".into(),
        server: "server-b".into(),
        session_id: "session-1".into(),
    }];
    assert_eq!(
        legacy_session_server(&tabs, "session-1", "server-a"),
        "server-b"
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn prefers_the_active_server_when_a_legacy_session_id_is_ambiguous() {
    let tabs = vec![
        Tab {
            kind: "session".into(),
            server: "server-a".into(),
            session_id: "session-1".into(),
        },
        Tab {
            kind: "session".into(),
            server: "server-b".into(),
            session_id: "session-1".into(),
        },
    ];
    assert_eq!(
        legacy_session_server(&tabs, "session-1", "server-b"),
        "server-b"
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn builds_and_decodes_a_server_keyed_session_route() {
    let server = "https://example.com:4096";
    let href = session_href(server, "session-1");
    assert_eq!(
        href,
        format!(
            "/server/{}/session/session-1",
            ts::encode_b64url(server.as_bytes())
        )
    );
    assert_eq!(
        require_server_key("aHR0cHM6Ly9leGFtcGxlLmNvbTo0MDk2"),
        Ok(server.to_string())
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn rejects_malformed_server_keys() {
    assert_eq!(
        require_server_key("not-base64"),
        Err("Invalid server route".to_string())
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn builds_the_legacy_directory_keyed_route() {
    assert_eq!(
        legacy_session_href("/Users/example/project", "session-1"),
        "/L1VzZXJzL2V4YW1wbGUvcHJvamVjdA/session/session-1"
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn resolves_the_root_session() {
    let mut sessions = HashMap::new();
    sessions.insert(
        "child".to_string(),
        Session {
            id: "child".into(),
            parent_id: Some("parent".into()),
        },
    );
    sessions.insert(
        "parent".to_string(),
        Session {
            id: "parent".into(),
            parent_id: Some("root".into()),
        },
    );
    sessions.insert(
        "root".to_string(),
        Session {
            id: "root".into(),
            parent_id: None,
        },
    );

    assert_eq!(
        root_session(sessions["child"].clone(), &sessions),
        Ok(Session {
            id: "root".into(),
            parent_id: None
        })
    );
}

#[test]
#[ignore = "porting: utils/session-route not implemented"]
fn rejects_a_parent_cycle() {
    let mut sessions = HashMap::new();
    sessions.insert(
        "child".to_string(),
        Session {
            id: "child".into(),
            parent_id: Some("parent".into()),
        },
    );
    sessions.insert(
        "parent".to_string(),
        Session {
            id: "parent".into(),
            parent_id: Some("child".into()),
        },
    );
    assert_eq!(
        root_session(sessions["child"].clone(), &sessions),
        Err("Session parent cycle: child".to_string())
    );
}
