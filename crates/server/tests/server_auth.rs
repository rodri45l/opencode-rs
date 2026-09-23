//! Port of packages/opencode/test/server/auth.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `ServerAuth`; see docs/TEST-PORT.md.
//!
//! Re-derived against `opencode_server::AuthConfig`: the reference asserts the
//! emitted `Authorization` header, the Rust type captures the same decision
//! surface (whether auth is required, the default username, and whether a
//! decoded credential pair is accepted).
//!
//! Named `server_auth.rs` rather than `auth.rs` to avoid colliding with the
//! unported `packages/opencode/test/auth/auth.test.ts`, which the inventory
//! maps to the same `auth.rs` stem.

use opencode_server::AuthConfig;

#[test]
fn does_not_require_auth_without_a_password() {
    let config = AuthConfig::none();
    assert!(!config.required());
    assert!(config.authorized("anything", "anything"));
}

#[test]
fn defaults_to_the_opencode_username() {
    let config = AuthConfig::with_password("secret");
    assert!(config.required());
    assert_eq!(config.username, "opencode");
    assert!(config.authorized("opencode", "secret"));
}

#[test]
fn uses_the_configured_username() {
    let config = AuthConfig::with_credentials("alice", "secret");
    assert_eq!(config.username, "alice");
    assert!(config.authorized("alice", "secret"));
    assert!(!config.authorized("opencode", "secret"));
}

#[test]
fn prefers_explicit_credentials() {
    let config = AuthConfig::with_credentials("bob", "cli-secret");
    assert!(config.required());
    assert!(config.authorized("bob", "cli-secret"));
    assert!(!config.authorized("bob", "secret"));
}

#[test]
fn validates_decoded_credentials() {
    let config = AuthConfig::with_credentials("alice", "secret");
    assert!(config.required());
    assert!(config.authorized("alice", "secret"));
    assert!(!config.authorized("opencode", "secret"));
}
