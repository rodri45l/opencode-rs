//! Header/URL helpers for the workspace proxy.
//!
//! Re-derived from `packages/opencode/src/server/proxy-util.ts` (upstream
//! 18ef3cc). The proxy forwards requests to a remote workspace target, so it
//! must rewrite websocket URLs, negotiate sub-protocols, and strip hop-by-hop
//! and opencode-internal headers before forwarding.

use axum::http::{HeaderMap, HeaderName, HeaderValue};

/// Headers that must not be forwarded to the proxy target.
const STRIPPED: &[&str] = &[
    "connection",
    "keep-alive",
    "transfer-encoding",
    "x-opencode-directory",
    "x-opencode-workspace",
    "accept-encoding",
];

/// Convert an HTTP target URL into its websocket equivalent.
///
/// `https://` becomes `wss://` and `http://` becomes `ws://`; the path, query,
/// and hash are preserved verbatim.
pub fn websocket_target_url(input: &str) -> String {
    if let Some(rest) = input.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = input.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        input.to_string()
    }
}

/// Parse the `sec-websocket-protocol` header into a trimmed, non-empty list.
pub fn websocket_protocols(headers: &HeaderMap) -> Vec<String> {
    headers
        .get("sec-websocket-protocol")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|protocol| !protocol.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Build the forwarded header set: copy non-stripped headers, then merge
/// `extra` (which overrides any existing value).
pub fn headers(input: &HeaderMap, extra: &[(&str, &str)]) -> HeaderMap {
    let mut out = HeaderMap::new();
    for (name, value) in input.iter() {
        if STRIPPED.contains(&name.as_str()) {
            continue;
        }
        out.append(name.clone(), value.clone());
    }
    for (name, value) in extra {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            out.insert(name, value);
        }
    }
    out
}
