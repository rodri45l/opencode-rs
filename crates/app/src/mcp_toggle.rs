//! MCP toggle ordering (port of packages/app/src/context/global-sync/mcp.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum McpStatus {
    Connected,
    NeedsAuth,
    Disabled,
    Pending,
}

#[derive(Default)]
pub struct McpCalls {
    pub calls: Vec<String>,
}

impl McpCalls {
    pub fn record(&mut self, name: &str) {
        self.calls.push(name.to_string());
    }
}

pub fn toggle_mcp(status: McpStatus, calls: &mut McpCalls) {
    match status {
        McpStatus::Pending => return,
        McpStatus::Connected => calls.record("disconnect"),
        McpStatus::NeedsAuth => calls.record("authenticate"),
        McpStatus::Disabled => calls.record("connect"),
    }
    calls.record("refresh");
}
