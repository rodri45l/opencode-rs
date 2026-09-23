//! Web search tool.
//!
//! Ports the observable behaviour of `packages/core/src/tool/websearch.ts`:
//! numeric controls are bounded, a provider is selected deterministically per
//! session (with an explicit operational override, Parallel before Exa), and
//! JSON-RPC responses parse from either a plain body or an SSE stream.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Maximum accepted `numResults`.
pub const MAX_NUM_RESULTS: u32 = 50;

/// Maximum accepted `contextMaxCharacters`.
pub const MAX_CONTEXT_CHARACTERS: usize = 200_000;

/// The legacy no-results fallback text.
pub const NO_RESULTS: &str = "No search results found. Please try a different query.";

/// The Exa MCP endpoint.
pub const EXA_URL: &str = "https://mcp.exa.ai/mcp";

/// The Parallel MCP endpoint.
pub const PARALLEL_URL: &str = "https://mcp.parallel.ai/mcp";

/// A decoded web search input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchInput {
    /// Search query.
    pub query: String,
    /// Optional number of results.
    pub num_results: Option<u32>,
    /// Optional context character budget.
    pub context_max_characters: Option<usize>,
}

/// Web search provider toggles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchConfig {
    /// Whether Exa is explicitly enabled.
    pub enable_exa: bool,
    /// Whether Parallel is explicitly enabled.
    pub enable_parallel: bool,
}

/// The web search tool.
#[derive(Debug, Default)]
pub struct WebSearchTool;

impl WebSearchTool {
    /// Decode and validate a web search input.
    pub fn parse_input(_value: &Value) -> CoreResult<WebSearchInput> {
        Err(CoreError::NotImplemented(
            "tool_websearch::WebSearchTool::parse_input",
        ))
    }

    /// Select a stable provider for a session, honoring explicit toggles.
    pub fn select_provider(
        _session_id: &str,
        _config: &SearchConfig,
        _explicit: Option<&str>,
    ) -> CoreResult<&'static str> {
        Err(CoreError::NotImplemented(
            "tool_websearch::WebSearchTool::select_provider",
        ))
    }

    /// Parse a JSON-RPC response body or SSE stream into result text.
    pub fn parse_response(_body: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_websearch::WebSearchTool::parse_response",
        ))
    }
}
