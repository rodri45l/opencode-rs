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
    pub fn parse_input(value: &Value) -> CoreResult<WebSearchInput> {
        let query = value
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("query is required".into()))?
            .to_string();
        if query.is_empty() {
            return Err(CoreError::Invalid("query is required".into()));
        }
        let num_results = match value.get("numResults") {
            Some(Value::Number(number)) => {
                let parsed = number.as_u64().ok_or_else(|| {
                    CoreError::Invalid("numResults must be a positive integer".into())
                })?;
                if parsed == 0 || parsed > MAX_NUM_RESULTS as u64 {
                    return Err(CoreError::Invalid(format!(
                        "numResults must be between 1 and {MAX_NUM_RESULTS}"
                    )));
                }
                Some(parsed as u32)
            }
            Some(Value::Null) | None => None,
            Some(_) => return Err(CoreError::Invalid("numResults must be a number".into())),
        };
        let context_max_characters = match value.get("contextMaxCharacters") {
            Some(Value::Number(number)) => {
                let parsed = number.as_u64().ok_or_else(|| {
                    CoreError::Invalid("contextMaxCharacters must be a positive integer".into())
                })?;
                if parsed > MAX_CONTEXT_CHARACTERS as u64 {
                    return Err(CoreError::Invalid(format!(
                        "contextMaxCharacters must be at most {MAX_CONTEXT_CHARACTERS}"
                    )));
                }
                Some(parsed as usize)
            }
            Some(Value::Null) | None => None,
            Some(_) => {
                return Err(CoreError::Invalid(
                    "contextMaxCharacters must be a number".into(),
                ))
            }
        };
        Ok(WebSearchInput {
            query,
            num_results,
            context_max_characters,
        })
    }

    /// Select a stable provider for a session, honoring explicit toggles.
    pub fn select_provider(
        session_id: &str,
        config: &SearchConfig,
        explicit: Option<&str>,
    ) -> CoreResult<&'static str> {
        if let Some(explicit) = explicit {
            return match explicit {
                "exa" => Ok("exa"),
                "parallel" => Ok("parallel"),
                other => Err(CoreError::Invalid(format!(
                    "unsupported search provider: {other}"
                ))),
            };
        }
        if config.enable_parallel {
            return Ok("parallel");
        }
        if config.enable_exa {
            return Ok("exa");
        }
        // Stable per session: an even hash picks parallel, otherwise exa.
        let hash = session_id.bytes().fold(0u64, |acc, byte| {
            acc.wrapping_mul(31).wrapping_add(byte as u64)
        });
        Ok(if hash % 2 == 0 { "parallel" } else { "exa" })
    }

    /// Parse a JSON-RPC response body or SSE stream into result text.
    pub fn parse_response(body: &str) -> CoreResult<String> {
        let payload = if body.contains("data:") {
            body.lines()
                .filter_map(|line| line.trim().strip_prefix("data:"))
                .map(str::trim)
                .find(|data| !data.is_empty() && *data != "[DONE]")
                .ok_or_else(|| CoreError::Invalid("no data payload in SSE stream".into()))?
        } else {
            body.trim()
        };
        let value: Value = serde_json::from_str(payload)
            .map_err(|error| CoreError::Invalid(format!("invalid JSON-RPC response: {error}")))?;
        let text = value
            .get("result")
            .and_then(|result| result.get("content"))
            .and_then(Value::as_array)
            .and_then(|content| {
                content
                    .iter()
                    .find_map(|part| part.get("text").and_then(Value::as_str))
            })
            .ok_or_else(|| CoreError::Invalid("response has no text content".into()))?;
        Ok(text.to_string())
    }
}
