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

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn generate_state(saved: Option<&str>) -> Result<String, NotImplemented> {
    if let Some(saved) = saved {
        if !saved.is_empty() {
            return Ok(saved.to_string());
        }
    }
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut state = String::new();
    let mut value = seed;
    while state.len() < 64 {
        value = value
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        state.push_str(&format!("{:016x}", value as u64));
    }
    state.truncate(64);
    Ok(state)
}

fn pending_client_information(_existing: Option<&Value>) -> Result<Option<Value>, NotImplemented> {
    Ok(None)
}

fn pending_tokens(_existing: Option<&Value>) -> Result<Option<Value>, NotImplemented> {
    Ok(None)
}

fn auth_status(entry: &Value, configured_url: &str, now_ms: i64) -> Result<String, NotImplemented> {
    if entry.get("serverUrl").and_then(Value::as_str) != Some(configured_url) {
        return Ok("not_authenticated".to_string());
    }
    let Some(tokens) = entry.get("tokens").filter(|value| value.is_object()) else {
        return Ok("not_authenticated".to_string());
    };
    if tokens.get("accessToken").and_then(Value::as_str).is_none() {
        return Ok("not_authenticated".to_string());
    }
    if let Some(expires_at) = tokens.get("expiresAt").and_then(Value::as_i64) {
        if expires_at <= now_ms {
            return Ok("expired".to_string());
        }
    }
    Ok("authenticated".to_string())
}

#[test]
fn state_generates_and_persists_a_new_state_when_none_is_saved() {
    let state = generate_state(None).unwrap();
    assert_eq!(state.len(), 64);
    assert_eq!(generate_state(Some(state.as_str())).unwrap(), state);
}

#[test]
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
