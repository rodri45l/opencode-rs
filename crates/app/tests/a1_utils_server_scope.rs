//! Port of packages/app/src/utils/server-scope.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use std::collections::BTreeMap;

use opencode_app::server_scope::{
    migrate_legacy_session_state_keys, route_from_route, scope_from_server_key, scoped_key_from,
    session_state_key, session_state_route,
};

#[test]
fn uses_a_stable_local_scope_for_the_canonical_sidecar() {
    assert_eq!(scope_from_server_key("sidecar", None), "local");
}

#[test]
fn keeps_configured_loopback_servers_distinct_from_the_canonical_sidecar() {
    assert_eq!(
        scope_from_server_key("http://localhost:4096", None),
        "http://localhost:4096"
    );
}

#[test]
fn uses_a_stable_local_scope_for_an_explicit_canonical_web_server() {
    let key = "http://localhost:4096";
    assert_eq!(scope_from_server_key(key, Some(key)), "local");
}

#[test]
fn combines_local_and_remote_scope_with_route_identity() {
    let route = route_from_route("cmVwbw", "session-1");
    assert_eq!(
        session_state_key("local", &route),
        "local\0cmVwbw/session-1"
    );
    assert_eq!(
        session_state_key("https://windows.example", &route),
        "https://windows.example\0cmVwbw/session-1"
    );
    assert_ne!(
        session_state_key("https://debian.example", &route),
        session_state_key("https://windows.example", &route)
    );
}

#[test]
fn extracts_route_keys_from_scoped_and_legacy_state_keys() {
    assert_eq!(session_state_route("cmVwbw/session-1"), "cmVwbw/session-1");
    assert_eq!(
        session_state_route("local\0cmVwbw/session-1"),
        "cmVwbw/session-1"
    );
    assert_eq!(
        session_state_route("https://debian.example\0cmVwbw/session-1"),
        "cmVwbw/session-1"
    );
}

#[test]
fn copies_legacy_route_keys_into_local_scope_without_overwriting_scoped_state() {
    let mut input = BTreeMap::new();
    input.insert("cmVwbw/session-1".to_string(), "legacy".to_string());
    input.insert("local\0cmVwbw/session-1".to_string(), "scoped".to_string());
    input.insert(
        "https://debian.example\0cmVwbw/session-1".to_string(),
        "remote".to_string(),
    );

    let mut expected = BTreeMap::new();
    expected.insert("local\0cmVwbw/session-1".to_string(), "scoped".to_string());
    expected.insert(
        "https://debian.example\0cmVwbw/session-1".to_string(),
        "remote".to_string(),
    );

    assert_eq!(migrate_legacy_session_state_keys(input), expected);
}

#[test]
fn rejects_invalid_identity_fragments() {
    assert_eq!(
        scoped_key_from("local", "bad\0directory"),
        Err("Scoped key part cannot contain null bytes".to_string())
    );
}
