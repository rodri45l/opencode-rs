//! Port of packages/opencode/test/mcp/oauth-auto-connect.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/mcp/oauth-provider.ts and src/mcp/index.ts; see
//! docs/TEST-PORT.md.
//!
//! Ported: the pure OAuth provider rules — `state()` generates a 64-char state
//! and persists it, the pending provider never exposes or overwrites existing
//! credentials before commit, and auth status is scoped to the configured server
//! URL (not_authenticated / authenticated / expired).
//! Dropped: every case that runs a real OAuth flow against a live MCP server
//! (needs_auth on first connect, failed reauth preserves credentials, successful
//! reauth commits replacements, authenticate connects resource-only servers).
//! Those need HTTP servers, the callback listener, and the Effect runtime.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn generate_state(_saved: Option<&str>) -> Result<String, NotImplemented> {
    nope("mcp oauth-auto-connect")
}

fn pending_client_information(_existing: Option<&Value>) -> Result<Option<Value>, NotImplemented> {
    nope("mcp oauth-auto-connect")
}

fn pending_tokens(_existing: Option<&Value>) -> Result<Option<Value>, NotImplemented> {
    nope("mcp oauth-auto-connect")
}

fn auth_status(
    _entry: &Value,
    _configured_url: &str,
    _now_ms: i64,
) -> Result<String, NotImplemented> {
    nope("mcp oauth-auto-connect")
}

#[test]
#[ignore = "porting: mcp oauth-auto-connect not implemented"]
fn state_generates_and_persists_a_new_state_when_none_is_saved() {
    let state = generate_state(None).unwrap();
    assert_eq!(state.len(), 64);
    assert_eq!(generate_state(Some(state.as_str())).unwrap(), state);
}

#[test]
#[ignore = "porting: mcp oauth-auto-connect not implemented"]
fn pending_provider_does_not_expose_or_overwrite_existing_credentials() {
    let existing_client = json!({ "clientId": "old-client" });
    let existing_tokens = json!({ "accessToken": "old-token" });
    assert_eq!(
        pending_client_information(Some(&existing_client)).unwrap(),
        None
    );
    assert_eq!(pending_tokens(Some(&existing_tokens)).unwrap(), None);
}

#[test]
#[ignore = "porting: mcp oauth-auto-connect not implemented"]
fn auth_status_only_reports_credentials_stored_for_the_configured_server_url() {
    let now = 1_000_000;
    let wrong_url = json!({
        "serverUrl": "https://old.example.com/mcp",
        "tokens": { "accessToken": "old-token" }
    });
    assert_eq!(
        auth_status(&wrong_url, "https://example.com/mcp", now).unwrap(),
        "not_authenticated"
    );

    let current = json!({
        "serverUrl": "https://example.com/mcp",
        "tokens": { "accessToken": "current-token" }
    });
    assert_eq!(
        auth_status(&current, "https://example.com/mcp", now).unwrap(),
        "authenticated"
    );

    let expired = json!({
        "serverUrl": "https://example.com/mcp",
        "tokens": { "accessToken": "expired-token", "expiresAt": 1 }
    });
    assert_eq!(
        auth_status(&expired, "https://example.com/mcp", now).unwrap(),
        "expired"
    );
}
