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
    pub fn parse_input(value: &Value) -> CoreResult<WebFetchInput> {
        let url = value
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("url is required".into()))?
            .to_string();
        if url.is_empty() {
            return Err(CoreError::Invalid("url is required".into()));
        }
        let format = value
            .get("format")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .unwrap_or("markdown")
            .to_string();
        let timeout = match value.get("timeout") {
            Some(Value::Number(number)) => {
                let timeout = number.as_u64().ok_or_else(|| {
                    CoreError::Invalid("timeout must be a positive integer".into())
                })?;
                if timeout == 0 || timeout > MAX_TIMEOUT_SECONDS {
                    return Err(CoreError::Invalid(format!(
                        "timeout must be between 1 and {MAX_TIMEOUT_SECONDS}"
                    )));
                }
                Some(timeout)
            }
            Some(Value::Null) | None => None,
            Some(_) => return Err(CoreError::Invalid("timeout must be a number".into())),
        };
        Ok(WebFetchInput {
            url,
            format,
            timeout,
        })
    }

    /// Extract visible text from HTML, dropping active content.
    pub fn extract_text_from_html(html: &str) -> CoreResult<String> {
        let stripped = strip_active(html);
        let mut out = String::new();
        let mut in_tag = false;
        for ch in stripped.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => out.push(ch),
                _ => {}
            }
        }
        let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
        Ok(collapsed)
    }

    /// Convert HTML to markdown, dropping active content.
    pub fn convert_html_to_markdown(html: &str) -> CoreResult<String> {
        let html = html.trim();
        let mut out = String::new();
        let mut rest = html;
        while !rest.is_empty() {
            if !rest.starts_with('<') {
                let end = rest.find('<').unwrap_or(rest.len());
                out.push_str(&unescape(&rest[..end]));
                rest = &rest[end..];
                continue;
            }
            let close = match rest.find('>') {
                Some(close) => close,
                None => {
                    out.push_str(rest);
                    break;
                }
            };
            let tag = rest[1..close].trim();
            rest = &rest[close + 1..];
            if let Some(closing) = tag.strip_prefix('/') {
                match closing
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str()
                {
                    "strong" | "b" => out.push_str("**"),
                    "em" | "i" => out.push('_'),
                    "h1" | "h2" | "h3" | "p" | "div" => out.push_str("\n\n"),
                    _ => {}
                }
                continue;
            }
            let name = tag.split_whitespace().next().unwrap_or("").to_lowercase();
            match name.as_str() {
                "h1" => out.push_str("# "),
                "h2" => out.push_str("## "),
                "h3" => out.push_str("### "),
                "strong" | "b" => out.push_str("**"),
                "em" | "i" => out.push('_'),
                "p" | "div" => out.push_str("\n\n"),
                "br" => out.push('\n'),
                name if name.starts_with("script") || name.starts_with("style") => {
                    let close_tag = format!("</{name}>");
                    if let Some(end) = rest.to_lowercase().find(&close_tag) {
                        rest = &rest[end + close_tag.len()..];
                    }
                }
                _ => {}
            }
        }
        let joined = out
            .split('\n')
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n");
        // Collapse runs of blank lines to a single blank line.
        let mut result = String::new();
        let mut blank_run = 0;
        for line in joined.split('\n') {
            if line.is_empty() {
                blank_run += 1;
                if blank_run > 1 {
                    continue;
                }
            } else {
                blank_run = 0;
            }
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(line);
        }
        Ok(result.trim().to_string())
    }

    /// Whether a URL uses a fetchable HTTP scheme.
    pub fn is_supported_scheme(url: &str) -> CoreResult<bool> {
        let lower = url.to_lowercase();
        Ok(lower.starts_with("http://") || lower.starts_with("https://"))
    }
}

fn strip_active(html: &str) -> String {
    let mut out = String::new();
    let lower = html.to_lowercase();
    let mut index = 0;
    while index < html.len() {
        let slice = &lower[index..];
        if let Some(start) = slice.find("<script") {
            let absolute = index + start;
            out.push_str(&html[index..absolute]);
            if let Some(end) = lower[absolute..].find("</script>") {
                index = absolute + end + "</script>".len();
            } else {
                index = html.len();
            }
            continue;
        }
        if let Some(start) = slice.find("<style") {
            let absolute = index + start;
            if let Some(script_start) = slice.find("<script") {
                if script_start < start {
                    continue;
                }
            }
            out.push_str(&html[index..absolute]);
            if let Some(end) = lower[absolute..].find("</style>") {
                index = absolute + end + "</style>".len();
            } else {
                index = html.len();
            }
            continue;
        }
        out.push_str(&html[index..]);
        break;
    }
    out
}

fn unescape(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
