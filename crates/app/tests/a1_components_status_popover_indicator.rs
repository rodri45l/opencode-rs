//! Port of packages/app/src/components/status-popover-indicator.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct StatusInput {
    ready: bool,
    server_health: Option<bool>,
    issue: bool,
    attention: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Services {
    mcp: Vec<String>,
    lsp: Vec<String>,
}

// Local stubs (fast wave): real module lands later.
fn server_status_dot_class(_input: StatusInput) -> String {
    String::new()
}

fn has_non_blocking_service_issue(_services: Services) -> bool {
    false
}

fn has_service_needing_attention(_services: Services) -> bool {
    false
}

#[test]
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
#[ignore = "porting: components/status-popover-indicator not implemented"]
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
