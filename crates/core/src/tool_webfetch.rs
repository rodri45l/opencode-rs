//! Web fetch tool.
//!
//! Ports the observable behaviour of `packages/core/src/tool/webfetch.ts`: the
//! input defaults `format` to `markdown` and bounds the timeout, HTML text and
//! markdown conversions strip active content (`script`/`style`), and only HTTP
//! schemes are fetched.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Maximum accepted timeout in seconds.
pub const MAX_TIMEOUT_SECONDS: u64 = 120;

/// Maximum accepted response size in bytes.
pub const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// A decoded web fetch input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebFetchInput {
    /// Requested URL.
    pub url: String,
    /// Requested output format.
    pub format: String,
    /// Optional timeout in seconds.
    pub timeout: Option<u64>,
}

/// The web fetch tool.
#[derive(Debug, Default)]
pub struct WebFetchTool;

impl WebFetchTool {
    /// Decode and validate a web fetch input.
    pub fn parse_input(_value: &Value) -> CoreResult<WebFetchInput> {
        Err(CoreError::NotImplemented(
            "tool_webfetch::WebFetchTool::parse_input",
        ))
    }

    /// Extract visible text from HTML, dropping active content.
    pub fn extract_text_from_html(_html: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_webfetch::WebFetchTool::extract_text_from_html",
        ))
    }

    /// Convert HTML to markdown, dropping active content.
    pub fn convert_html_to_markdown(_html: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_webfetch::WebFetchTool::convert_html_to_markdown",
        ))
    }

    /// Whether a URL uses a fetchable HTTP scheme.
    pub fn is_supported_scheme(_url: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "tool_webfetch::WebFetchTool::is_supported_scheme",
        ))
    }
}
