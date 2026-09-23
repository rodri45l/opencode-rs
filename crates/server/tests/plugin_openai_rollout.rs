//! Port of packages/opencode/test/plugin/openai-rollout.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the experimental WebSocket transport is on by default for
//! pre-release channels and opt-in for releases.

use opencode_server::port::plugin::experimental_websockets_enabled;

#[test]
fn enables_websockets_by_default_only_on_pre_release_channels() {
    assert!(experimental_websockets_enabled(false, Some("local")));
    assert!(experimental_websockets_enabled(false, Some("dev")));
    assert!(experimental_websockets_enabled(false, Some("beta")));
    assert!(!experimental_websockets_enabled(false, Some("latest")));
    assert!(!experimental_websockets_enabled(false, Some("prod")));
}

#[test]
fn allows_releases_to_opt_in_through_the_experimental_flag() {
    assert!(experimental_websockets_enabled(true, Some("latest")));
    assert!(experimental_websockets_enabled(true, Some("prod")));
}
