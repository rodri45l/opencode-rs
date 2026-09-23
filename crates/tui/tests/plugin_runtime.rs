//! Port of packages/tui/test/plugin/runtime.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/plugin/runtime.tsx; see docs/TEST-PORT.md.

use opencode_tui::plugin_runtime::{create_plugin_runtime, PluginCommands, Route, StatusEntry};

struct TestCommands;

impl PluginCommands for TestCommands {
    fn activate(&self, id: &str) -> bool {
        id == "demo"
    }
}

fn first() -> &'static str {
    "first"
}

fn second() -> &'static str {
    "second"
}

fn status_entry() -> StatusEntry {
    StatusEntry {
        id: "demo".to_string(),
        source: "internal".to_string(),
        spec: "demo".to_string(),
        target: "demo".to_string(),
        enabled: true,
        active: true,
    }
}

#[test]
fn routes_use_the_latest_registration_and_restore_previous_registrations() {
    let runtime = create_plugin_runtime::<fn() -> &'static str, TestCommands>();
    let _ = runtime.routes.register(vec![Route {
        name: "demo".to_string(),
        render: first,
    }]);
    let dispose = runtime.routes.register(vec![Route {
        name: "demo".to_string(),
        render: second,
    }]);

    assert_eq!(
        runtime.routes.get("demo").map(|render| render()),
        Some("second")
    );
    dispose();
    assert_eq!(
        runtime.routes.get("demo").map(|render| render()),
        Some("first")
    );
}

#[test]
fn facade_publishes_and_clears_presentation_state() {
    let mut runtime = create_plugin_runtime::<fn() -> &'static str, TestCommands>();
    runtime.update(Some(TestCommands), vec![status_entry()]);

    assert!(runtime.commands().activate("demo"));
    assert_eq!(runtime.status().len(), 1);
    runtime.clear();
    assert!(!runtime.commands().activate("demo"));
    assert!(runtime.status().is_empty());
}
