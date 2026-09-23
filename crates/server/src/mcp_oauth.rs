//! MCP OAuth callback helpers.
//!
//! Re-derived from `packages/opencode/src/mcp/oauth-provider.ts` and
//! `packages/opencode/src/mcp/oauth-callback.ts` (upstream 18ef3cc). The
//! loopback callback server itself is not ported yet; this module carries the
//! pure pieces: redirect-URI parsing and the branded callback pages.

use url::Url;

/// Default loopback port for the MCP OAuth callback.
pub const OAUTH_CALLBACK_PORT: u16 = 19876;

/// Default callback path.
pub const OAUTH_CALLBACK_PATH: &str = "/mcp/oauth/callback";

/// The port and path a redirect URI resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectUri {
    pub port: u16,
    pub path: String,
}

/// Resolve the callback port and path for a redirect URI.
///
/// With no URI, or an unparsable one, the defaults are returned. A missing
/// port falls back to `443` for `https:` and `80` otherwise.
pub fn parse_redirect_uri(redirect_uri: Option<&str>) -> RedirectUri {
    let Some(input) = redirect_uri else {
        return defaults();
    };
    match Url::parse(input) {
        Ok(url) => {
            let port = url
                .port()
                .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
            let path = if url.path().is_empty() {
                OAUTH_CALLBACK_PATH.to_string()
            } else {
                url.path().to_string()
            };
            RedirectUri { port, path }
        }
        Err(_) => defaults(),
    }
}

fn defaults() -> RedirectUri {
    RedirectUri {
        port: OAUTH_CALLBACK_PORT,
        path: OAUTH_CALLBACK_PATH.to_string(),
    }
}

/// Escape a value for safe interpolation into HTML text or an attribute.
pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// The "authorization successful" callback page.
pub fn success_page(provider: Option<&str>) -> String {
    let message = match provider {
        Some(provider) => format!("OpenCode is now connected to {}.", escape_html(provider)),
        None => "OpenCode is now authorized.".to_string(),
    };
    render_document(
        "Authorization successful",
        &render_card("success", "Authorization successful", &message, None),
    )
}

/// The "authorization failed" callback page, with an optional detail line.
pub fn error_page(detail: &str, provider: Option<&str>) -> String {
    let message = match provider {
        Some(provider) => format!(
            "OpenCode couldn't finish connecting to {}.",
            escape_html(provider)
        ),
        None => "OpenCode couldn't complete authorization.".to_string(),
    };
    render_document(
        "Authorization failed",
        &render_card("error", "Authorization failed", &message, Some(detail)),
    )
}

fn render_card(status: &str, headline: &str, message: &str, detail: Option<&str>) -> String {
    let detail = detail.map(str::trim).unwrap_or("");
    let detail_block = if detail.is_empty() {
        r#"<pre class="detail" id="oc-detail" hidden></pre>"#.to_string()
    } else {
        format!(
            r#"<pre class="detail" id="oc-detail">{}</pre>"#,
            escape_html(detail)
        )
    };
    format!(
        r#"<main class="card" id="oc-card" data-status="{status}" role="status" aria-live="polite"><h1 class="headline" id="oc-headline">{headline}</h1><p class="message" id="oc-message">{message}</p>{detail_block}</main>"#
    )
}

fn render_document(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" /><meta name="robots" content="noindex" /><title>{title} · OpenCode</title></head><body>{body}</body></html>"#
    )
}
