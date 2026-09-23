//! Server-action referer sanitization.
//!
//! Port of `packages/console/app/src/lib/server-action.ts` (upstream 18ef3cc):
//! a `/_server` referer is preserved when it is same-origin and replaced with
//! the request origin otherwise; other routes are untouched.

use url::Url;

/// The sanitized request view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedRequest {
    pub url: String,
    pub referer: String,
    pub changed: bool,
}

/// Sanitize the referer of a server-action request.
pub fn sanitize_server_action_request(url: &str, referer: Option<&str>) -> SanitizedRequest {
    let parsed = Url::parse(url).ok();
    let pathname = parsed.as_ref().map(|u| u.path()).unwrap_or("");
    if pathname != "/_server" {
        return SanitizedRequest {
            url: url.to_string(),
            referer: referer.unwrap_or("").to_string(),
            changed: false,
        };
    }
    let origin = parsed
        .as_ref()
        .map(|u| u.origin().ascii_serialization())
        .unwrap_or_default();
    if let Some(referer) = referer {
        if let Ok(referer_url) = Url::parse(referer) {
            let same_origin = parsed
                .as_ref()
                .map(|u| referer_url.origin() == u.origin())
                .unwrap_or(false);
            if same_origin {
                return SanitizedRequest {
                    url: url.to_string(),
                    referer: referer.to_string(),
                    changed: false,
                };
            }
        }
    }
    SanitizedRequest {
        url: url.to_string(),
        referer: origin,
        changed: true,
    }
}
