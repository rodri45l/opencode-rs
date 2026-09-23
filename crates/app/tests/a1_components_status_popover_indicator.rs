//! Port of packages/app/src/components/status-popover-indicator.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::status_popover_indicator::{
    has_non_blocking_service_issue, has_service_needing_attention, server_status_dot_class,
    Services, StatusInput,
};

#[test]
fn uses_the_success_token_while_the_server_and_services_are_healthy() {
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: true,
            server_health: Some(true),
            issue: false,
            attention: false
        }),
        "bg-icon-success-base"
    );
}

#[test]
fn uses_the_session_attention_token_when_a_service_needs_attention() {
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: true,
            server_health: Some(true),
            issue: true,
            attention: true
        }),
        "bg-v2-background-bg-accent"
    );
}

#[test]
fn uses_the_warning_token_for_non_blocking_issues_while_the_server_is_online() {
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: true,
            server_health: Some(true),
            issue: true,
            attention: false
        }),
        "bg-icon-warning-base"
    );
}

#[test]
fn uses_the_critical_token_only_after_the_server_connection_drops() {
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: true,
            server_health: Some(false),
            issue: false,
            attention: false
        }),
        "bg-icon-critical-base"
    );
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: true,
            server_health: Some(false),
            issue: true,
            attention: false
        }),
        "bg-icon-critical-base"
    );
}

#[test]
fn stays_neutral_before_status_is_ready() {
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: false,
            server_health: Some(true),
            issue: false,
            attention: false
        }),
        "bg-border-weak-base"
    );
    assert_eq!(
        server_status_dot_class(StatusInput {
            ready: false,
            server_health: None,
            issue: false,
            attention: false
        }),
        "bg-border-weak-base"
    );
}

#[test]
fn detects_mcp_failures_that_do_not_block_chatting() {
    assert!(has_non_blocking_service_issue(Services {
        mcp: vec!["failed".into()],
        lsp: Vec::new()
    }));
    assert!(has_non_blocking_service_issue(Services {
        mcp: vec!["needs_auth".into()],
        lsp: Vec::new()
    }));
    assert!(has_non_blocking_service_issue(Services {
        mcp: vec!["needs_client_registration".into()],
        lsp: Vec::new()
    }));
    assert!(!has_non_blocking_service_issue(Services {
        mcp: vec!["connected".into(), "pending".into(), "disabled".into()],
        lsp: Vec::new()
    }));
}

#[test]
fn detects_lsp_failures_that_do_not_block_chatting() {
    assert!(has_non_blocking_service_issue(Services {
        mcp: Vec::new(),
        lsp: vec!["error".into()]
    }));
    assert!(!has_non_blocking_service_issue(Services {
        mcp: Vec::new(),
        lsp: vec!["connected".into()]
    }));
}

#[test]
fn detects_mcp_states_that_need_user_attention() {
    assert!(has_service_needing_attention(Services {
        mcp: vec!["needs_auth".into()],
        lsp: Vec::new()
    }));
    assert!(has_service_needing_attention(Services {
        mcp: vec!["needs_client_registration".into()],
        lsp: Vec::new()
    }));
}

#[test]
fn ignores_states_that_do_not_need_user_attention() {
    assert!(!has_service_needing_attention(Services {
        mcp: vec!["failed".into()],
        lsp: Vec::new()
    }));
    assert!(!has_service_needing_attention(Services {
        mcp: vec!["connected".into(), "pending".into(), "disabled".into()],
        lsp: Vec::new()
    }));
}
