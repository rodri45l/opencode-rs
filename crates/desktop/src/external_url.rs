//! External URL and local file URL resolution.
//!
//! Port of `packages/desktop/src/main/external-url.ts` (upstream 18ef3cc).

use url::Url;

/// Resolve an externally-openable URL (`http`, `https`, `mailto`).
pub fn resolve_external_url(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    match url.scheme() {
        "http" | "https" | "mailto" => Some(url.as_str().to_string()),
        _ => None,
    }
}

/// Resolve a local `file:` URL to a filesystem path.
pub fn resolve_local_file_path(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    if url.host_str().map(|host| !host.is_empty()).unwrap_or(false) {
        return None;
    }
    url.to_file_path()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}
