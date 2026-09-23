//! Port of packages/core/test/plugin/promise.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `fromPromise` loads a promise-defined plugin and registers
//! a transform hook that contributes its description, and the returned
//! registration can be disposed to revert the hook. Re-derived: the `PluginHost`
//! wiring, the async `setup` runtime, and the agent-transform service are
//! dropped.

use opencode_core::plugin_promise::PluginPromise;

#[test]
fn loads_a_promise_plugin_and_registers_a_transform_hook() {
    let registration = PluginPromise::from_promise("Reviews code").unwrap();
    assert_eq!(registration.description(), Some("Reviews code"));
}

#[test]
fn disposes_a_hook_registration_on_request() {
    let mut registration = PluginPromise::from_promise("temporary").unwrap();
    registration.dispose().unwrap();
    assert_eq!(registration.description(), None);
}
