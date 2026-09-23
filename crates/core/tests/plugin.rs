//! Port of packages/core/test/plugin.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: plugins are keyed by id; adding an id applies its
//! contribution, adding the same id again replaces the previous contribution,
//! and removing the id reverts it. Re-derived: the `PluginV2` service, `wait`
//! semantics, activation-defect propagation, and the agent-transform wiring are
//! dropped.

use opencode_core::plugin::PluginRegistry;

const NOTE: &str = "porting: plugin registry not implemented";

#[test]
fn adds_replaces_and_removes_plugins() {
    let mut registry = PluginRegistry::new();

    registry.add("managed", "first").expect(NOTE);
    assert_eq!(
        registry
            .applied_description("managed")
            .expect(NOTE)
            .as_deref(),
        Some("first")
    );

    registry.add("managed", "second").expect(NOTE);
    assert_eq!(
        registry
            .applied_description("managed")
            .expect(NOTE)
            .as_deref(),
        Some("second")
    );

    registry.remove("managed").expect(NOTE);
    assert_eq!(registry.applied_description("managed").expect(NOTE), None);
}
