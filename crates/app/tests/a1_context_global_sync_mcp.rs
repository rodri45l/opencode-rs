//! Port of packages/app/src/context/global-sync/mcp.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::mcp_toggle::{toggle_mcp, McpCalls, McpStatus};

#[test]
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
fn does_not_toggle_a_server_while_its_connection_is_pending() {
    let mut pending = McpCalls::default();
    toggle_mcp(McpStatus::Pending, &mut pending);
    assert!(pending.calls.is_empty());
}
