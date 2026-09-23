//! Port of packages/app/src/utils/session-route.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use std::collections::HashMap;

use opencode_app::session_route::{
    legacy_session_href, legacy_session_server, require_server_key, root_session, session_href,
    Session, Tab,
};
use opencode_test_support as ts;

#[test]
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
fn rejects_malformed_server_keys() {
    assert_eq!(
        require_server_key("not-base64"),
        Err("Invalid server route".to_string())
    );
}

#[test]
fn builds_the_legacy_directory_keyed_route() {
    assert_eq!(
        legacy_session_href("/Users/example/project", "session-1"),
        "/L1VzZXJzL2V4YW1wbGUvcHJvamVjdA/session/session-1"
    );
}

#[test]
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
