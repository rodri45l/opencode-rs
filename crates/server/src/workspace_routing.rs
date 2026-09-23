//! Workspace routing decisions shared by the control plane and the proxy.
//!
//! Re-derived from `packages/opencode/src/server/shared/workspace-routing.ts`
//! (upstream 18ef3cc). The reference classifies a request as local or forwarded,
//! extracts the session id from a workspace-route path, and rewrites a proxy URL
//! so the remote resolves against its own root.

use opencode_schema::SessionId;
use url::Url;

/// One routing rule. `method` is unset when the rule applies to every method.
struct Rule {
    method: Option<&'static str>,
    path: &'static str,
    local: bool,
}

const RULES: &[Rule] = &[
    Rule {
        method: None,
        path: "/experimental/workspace",
        local: true,
    },
    Rule {
        method: None,
        path: "/session/status",
        local: false,
    },
    Rule {
        method: Some("GET"),
        path: "/session",
        local: true,
    },
];

/// Whether a request stays on the control plane rather than being forwarded to
/// a selected workspace target. Rules are matched in declaration order; a
/// non-exact rule matches the path or any path nested beneath it.
pub fn is_local_workspace_route(method: &str, path: &str) -> bool {
    for rule in RULES {
        if let Some(required) = rule.method {
            if required != method {
                continue;
            }
        }
        if path == rule.path
            || path.starts_with(rule.path) && path.as_bytes().get(rule.path.len()) == Some(&b'/')
        {
            return rule.local;
        }
    }
    false
}

/// Extract the session id from a workspace-route path.
///
/// Matches `/session/<id>` (with an optional trailing path) and
/// `/experimental/session/<id>/background`. `/session/status` and non-session
/// paths yield `None`.
pub fn get_workspace_route_session_id(url: &Url) -> Option<SessionId> {
    let path = url.path();
    if path == "/session/status" {
        return None;
    }

    let id = segment_after(path, "/session/")
        .map(|(id, _)| id)
        .or_else(|| {
            let (id, rest) = segment_after(path, "/experimental/session/")?;
            (rest == "/background").then_some(id)
        })?;

    SessionId::parse(id).ok()
}

/// Split `path` after `prefix` into its first segment and the remaining suffix.
fn segment_after<'a>(path: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    let rest = path.strip_prefix(prefix)?;
    let end = rest.find('/').unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some((&rest[..end], &rest[end..]))
}

/// Rewrite a proxy target for a request: append the request path to the target
/// base, preserve the request query (minus `workspace` and `directory`) and
/// fragment, and strip any query the target itself carried.
pub fn workspace_proxy_url(target: &str, request: &Url) -> Result<Url, url::ParseError> {
    let mut proxy = Url::parse(target)?;
    let base = proxy.path().trim_end_matches('/');
    proxy.set_path(&format!("{base}{}", request.path()));

    let pairs: Vec<(String, String)> = request
        .query_pairs()
        .filter(|(key, _)| key.as_ref() != "workspace" && key.as_ref() != "directory")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if pairs.is_empty() {
        proxy.set_query(None);
    } else {
        proxy.query_pairs_mut().clear().extend_pairs(pairs);
    }

    proxy.set_fragment(request.fragment());
    Ok(proxy)
}
