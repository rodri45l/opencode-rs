//! Status popover indicator (port of packages/app/src/components/status-popover-indicator.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct StatusInput {
    pub ready: bool,
    pub server_health: Option<bool>,
    pub issue: bool,
    pub attention: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Services {
    pub mcp: Vec<String>,
    pub lsp: Vec<String>,
}

pub fn server_status_dot_class(input: StatusInput) -> String {
    if input.server_health == Some(false) {
        return "bg-icon-critical-base".to_string();
    }
    if !input.ready || input.server_health.is_none() {
        return "bg-border-weak-base".to_string();
    }
    if input.attention {
        return "bg-v2-background-bg-accent".to_string();
    }
    if input.issue {
        return "bg-icon-warning-base".to_string();
    }
    if input.server_health == Some(true) {
        return "bg-icon-success-base".to_string();
    }
    "bg-border-weak-base".to_string()
}

pub fn has_non_blocking_service_issue(services: Services) -> bool {
    let mcp_issue = services
        .mcp
        .iter()
        .any(|status| status != "connected" && status != "pending" && status != "disabled");
    let lsp_issue = services.lsp.iter().any(|status| status == "error");
    mcp_issue || lsp_issue
}

pub fn has_service_needing_attention(services: Services) -> bool {
    services
        .mcp
        .iter()
        .any(|status| status == "needs_auth" || status == "needs_client_registration")
}
