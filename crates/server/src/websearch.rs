//! Web-search provider selection and MCP response parsing.
//!
//! Ports the observable behaviour of `packages/opencode/src/tool/websearch.ts`
//! and `tool/mcp-websearch.ts`: a stable per-session provider choice with an
//! env override and feature flags, branded labels, the analytics model id, and
//! the JSON-RPC/SSE response parser.

use serde_json::Value;

use crate::tools::ToolError;

/// Feature flags that pin a web-search provider.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WebSearchFlags {
    /// Route to Exa.
    pub exa: bool,
    /// Route to Parallel.
    pub parallel: bool,
}

/// Select the web-search provider for a session.
pub fn select_web_search_provider(session_id: &str, flags: WebSearchFlags) -> String {
    if let Ok(value) = std::env::var("OPENCODE_WEBSEARCH_PROVIDER") {
        if value == "parallel" || value == "exa" {
            return value;
        }
    }
    if flags.exa {
        return "exa".to_string();
    }
    if flags.parallel {
        return "parallel".to_string();
    }
    let checksum: u32 = session_id.bytes().map(u32::from).sum();
    if checksum % 2 == 0 {
        "exa".to_string()
    } else {
        "parallel".to_string()
    }
}

/// Whether web search is enabled for `provider_id`.
pub fn web_search_enabled(provider_id: &str, flags: WebSearchFlags) -> bool {
    provider_id == "opencode" || provider_id.starts_with("opencode-") || flags.exa || flags.parallel
}

/// The branded provider label.
pub fn web_search_provider_label(provider: Option<&str>) -> String {
    match provider {
        Some("parallel") => "Parallel Web Search".to_string(),
        Some("exa") => "Exa Web Search".to_string(),
        _ => "Web Search".to_string(),
    }
}

/// The provider API model id used for analytics.
pub fn web_search_model_name(api_id: &str) -> String {
    api_id.to_string()
}

/// Parse a JSON-RPC (or SSE-framed) MCP response into its text content.
pub fn parse_response(payload: &str) -> Result<String, ToolError> {
    if let Some(text) = extract_jsonrpc_text(payload) {
        return Ok(text);
    }
    for line in payload.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            if let Some(text) = extract_jsonrpc_text(data.trim()) {
                return Ok(text);
            }
        }
    }
    Err(ToolError::Message(
        "no JSON-RPC text content in response".to_string(),
    ))
}

fn extract_jsonrpc_text(raw: &str) -> Option<String> {
    let value: Value = serde_json::from_str(raw).ok()?;
    let content = value.get("result")?.get("content")?.as_array()?;
    for part in content {
        if part.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                return Some(text.to_string());
            }
        }
    }
    None
}
