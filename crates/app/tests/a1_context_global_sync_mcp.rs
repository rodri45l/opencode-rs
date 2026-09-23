//! Port of packages/app/src/context/global-sync/mcp.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
enum McpStatus {
    Connected,
    NeedsAuth,
    Disabled,
    Pending,
}

#[derive(Default)]
struct McpCalls {
    calls: Vec<String>,
}

// Local stub (fast wave): real module lands later.
fn toggle_mcp(_status: McpStatus, _calls: &mut McpCalls) {}

impl McpCalls {
    fn record(&mut self, name: &str) {
        self.calls.push(name.to_string());
    }
}

#[test]
#[ignore = "porting: context/global-sync/mcp not implemented"]
fn runs_the_status_action_before_refreshing_the_owning_query() {
    let mut connected = McpCalls::default();
    toggle_mcp(McpStatus::Connected, &mut connected);
    assert_eq!(
        connected.calls,
        vec!["disconnect".to_string(), "refresh".to_string()]
    );

    let mut needs_auth = McpCalls::default();
    toggle_mcp(McpStatus::NeedsAuth, &mut needs_auth);
    assert_eq!(
        needs_auth.calls,
        vec!["authenticate".to_string(), "refresh".to_string()]
    );

    let mut disabled = McpCalls::default();
    toggle_mcp(McpStatus::Disabled, &mut disabled);
    assert_eq!(
        disabled.calls,
        vec!["connect".to_string(), "refresh".to_string()]
    );
}

#[test]
#[ignore = "porting: context/global-sync/mcp not implemented"]
fn does_not_toggle_a_server_while_its_connection_is_pending() {
    let mut pending = McpCalls::default();
    toggle_mcp(McpStatus::Pending, &mut pending);
    assert!(pending.calls.is_empty());
}
