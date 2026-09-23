//! Authorization redirect allow-listing.
//!
//! Port of `packages/console/function/src/auth-redirect.ts` (upstream 18ef3cc):
//! only the registered `app` client may redirect to an OpenCode callback host.

use url::Url;

/// Whether `client` may be redirected to `redirect_uri`.
pub fn is_allowed_authorization_redirect(client: &str, redirect_uri: &str) -> bool {
    if client != "app" {
        return false;
    }
    let Ok(redirect) = Url::parse(redirect_uri) else {
        return false;
    };
    let host = redirect.host_str().unwrap_or("");
    if host == "localhost" || host == "127.0.0.1" {
        return matches!(redirect.scheme(), "http" | "https");
    }
    redirect.scheme() == "https" && (host == "opencode.ai" || host.ends_with(".opencode.ai"))
}
