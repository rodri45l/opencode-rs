//! Port of packages/core/test/plugin/promise.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `fromPromise` loads a promise-defined plugin and registers
//! a transform hook that contributes its description, and the returned
//! registration can be disposed to revert the hook. Re-derived: the `PluginHost`
//! wiring, the async `setup` runtime, and the agent-transform service are
//! dropped.

use opencode_core::plugin_promise::PluginPromise;

const NOTE: &str = "porting: promise plugin adapter not implemented";

#[test]
#[ignore = "porting: promise plugin adapter not implemented"]
fn loads_a_promise_plugin_and_registers_a_transform_hook() {
    let registration = PluginPromise::from_promise("Reviews code").expect(NOTE);
    assert_eq!(registration.description(), Some("Reviews code"));
}

#[test]
#[ignore = "porting: promise plugin adapter not implemented"]
fn disposes_a_hook_registration_on_request() {
    let mut registration = PluginPromise::from_promise("temporary").expect(NOTE);
    registration.dispose().expect(NOTE);
    assert_eq!(registration.description(), None);
}
