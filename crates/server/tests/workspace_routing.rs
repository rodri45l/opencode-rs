//! Port of packages/opencode/test/server/workspace-routing.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `src/server/shared/workspace-routing.ts`; see docs/TEST-PORT.md.
//!
//! Re-derived: the reference returns a `SessionID`; the Rust surface returns the
//! prefix-validated `opencode_schema::SessionId`.

use opencode_schema::SessionId;
use opencode_server::workspace_routing::{
    get_workspace_route_session_id, is_local_workspace_route, workspace_proxy_url,
};
use url::Url;

fn url(input: &str) -> Url {
    Url::parse(input).expect("url")
}

fn session_id(input: &str) -> SessionId {
    SessionId::parse(input).expect("session id")
}

fn query(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(name, _)| name.as_ref() == key)
        .map(|(_, value)| value.into_owned())
}

#[test]
fn get_session_is_local() {
    assert!(is_local_workspace_route("GET", "/session"));
}

#[test]
fn get_session_with_id_is_local_by_prefix() {
    assert!(is_local_workspace_route("GET", "/session/ses_abc"));
}

#[test]
fn post_session_is_not_local_by_method() {
    assert!(!is_local_workspace_route("POST", "/session"));
}

#[test]
fn session_status_is_forwarded_regardless_of_method() {
    assert!(!is_local_workspace_route("GET", "/session/status"));
    assert!(!is_local_workspace_route("POST", "/session/status"));
}

#[test]
fn unrecognized_paths_are_not_local() {
    assert!(!is_local_workspace_route("GET", "/config"));
    assert!(!is_local_workspace_route(
        "POST",
        "/session/ses_abc/message"
    ));
}

#[test]
fn extracts_session_id_from_path() {
    assert_eq!(
        get_workspace_route_session_id(&url("http://localhost/session/ses_abc123/message")),
        Some(session_id("ses_abc123"))
    );
}

#[test]
fn extracts_session_id_without_trailing_path() {
    assert_eq!(
        get_workspace_route_session_id(&url("http://localhost/session/ses_xyz")),
        Some(session_id("ses_xyz"))
    );
}

#[test]
fn extracts_session_id_from_experimental_background_path() {
    assert_eq!(
        get_workspace_route_session_id(&url(
            "http://localhost/experimental/session/ses_bg/background"
        )),
        Some(session_id("ses_bg"))
    );
}

#[test]
fn returns_none_for_session_status() {
    assert_eq!(
        get_workspace_route_session_id(&url("http://localhost/session/status")),
        None
    );
}

#[test]
fn returns_none_for_non_session_paths() {
    assert_eq!(
        get_workspace_route_session_id(&url("http://localhost/config")),
        None
    );
}

#[test]
fn returns_none_for_bare_session_path() {
    assert_eq!(
        get_workspace_route_session_id(&url("http://localhost/session")),
        None
    );
}

#[test]
fn proxy_url_appends_request_path_to_target() {
    let result = workspace_proxy_url("http://remote:8080/base", &url("http://localhost/config"))
        .expect("proxy url");
    assert_eq!(result.as_str(), "http://remote:8080/base/config");
}

#[test]
fn proxy_url_strips_trailing_slash_on_target() {
    let result = workspace_proxy_url(
        "http://remote:8080/base/",
        &url("http://localhost/session/abc"),
    )
    .expect("proxy url");
    assert_eq!(result.path(), "/base/session/abc");
}

#[test]
fn proxy_url_preserves_query_params_but_removes_workspace() {
    let request = url("http://localhost/config?workspace=ws_123&keep=yes");
    let result = workspace_proxy_url("http://remote:8080/base", &request).expect("proxy url");
    assert_eq!(query(&result, "workspace"), None);
    assert_eq!(query(&result, "keep"), Some("yes".to_string()));
}

#[test]
fn proxy_url_strips_the_host_directory_param() {
    let request = url("http://localhost/session/abc?directory=F%3A%5Cproj&keep=yes");
    let result = workspace_proxy_url("http://remote:8080/base", &request).expect("proxy url");
    assert_eq!(query(&result, "directory"), None);
    assert_eq!(query(&result, "keep"), Some("yes".to_string()));
}

#[test]
fn proxy_url_preserves_hash_from_request() {
    let result = workspace_proxy_url("http://remote:8080", &url("http://localhost/page#section"))
        .expect("proxy url");
    assert_eq!(result.fragment(), Some("section"));
}

#[test]
fn proxy_url_accepts_url_target() {
    let target = url("http://remote:3000/api");
    let result =
        workspace_proxy_url(target.as_str(), &url("http://localhost/users")).expect("proxy url");
    assert_eq!(result.as_str(), "http://remote:3000/api/users");
}
