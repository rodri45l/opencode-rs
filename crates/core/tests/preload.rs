//! Port of packages/core/test/preload.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the preload disables public npm security audits by setting
//! `NPM_CONFIG_AUDIT=false`. Re-derived: the reference test asserts the process
//! environment directly; the Rust check asserts the preload environment bundle.

use opencode_core::preload::Preload;

const NOTE: &str = "porting: preload not implemented";

#[test]
fn disables_public_npm_security_audits() {
    let environment = Preload::environment().expect(NOTE);
    assert!(environment
        .iter()
        .any(|(key, value)| key == "NPM_CONFIG_AUDIT" && value == "false"));
}
