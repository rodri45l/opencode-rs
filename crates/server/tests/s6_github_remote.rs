//! Port of packages/opencode/test/cli/github-remote.test.ts (upstream 18ef3cc).
//!
//! RED-first: `cli/cmd/github`'s `parseGitHubRemote` lives in the CLI crate; no
//! server API implements it yet, so the reference behaviour is pinned against a
//! local typed stub (returns `None`) that the integration fixer replaces with
//! the crate's real API.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitHubRemote {
    owner: String,
    repo: String,
}

fn remote(owner: &str, repo: &str) -> GitHubRemote {
    GitHubRemote {
        owner: owner.to_string(),
        repo: repo.to_string(),
    }
}

/// Stub for `parseGitHubRemote`. Deliberately unimplemented: all cases below are
/// red until the real helper lands.
fn parse_github_remote(_url: &str) -> Option<GitHubRemote> {
    None
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_https_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("https://github.com/sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_https_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("https://github.com/sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_git_at_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("git@github.com:sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_git_at_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("git@github.com:sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_ssh_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("ssh://git@github.com/sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_ssh_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("ssh://git@github.com/sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_git_protocol_urls_from_package_metadata() {
    assert_eq!(
        parse_github_remote("git://github.com/facebook/react.git"),
        Some(remote("facebook", "react"))
    );
    assert_eq!(
        parse_github_remote("git+https://github.com/facebook/react.git"),
        Some(remote("facebook", "react"))
    );
    assert_eq!(
        parse_github_remote("git+ssh://git@github.com/facebook/react.git"),
        Some(remote("facebook", "react"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn npm_style_github_shorthand_is_null() {
    assert_eq!(parse_github_remote("github:facebook/react"), None);
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_http_url() {
    assert_eq!(
        parse_github_remote("http://github.com/owner/repo"),
        Some(remote("owner", "repo"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_hyphenated_names() {
    assert_eq!(
        parse_github_remote("https://github.com/my-org/my-repo.git"),
        Some(remote("my-org", "my-repo"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_underscore_names() {
    assert_eq!(
        parse_github_remote("git@github.com:my_org/my_repo.git"),
        Some(remote("my_org", "my_repo"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_numeric_names() {
    assert_eq!(
        parse_github_remote("https://github.com/org123/repo456"),
        Some(remote("org123", "repo456"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn parses_dotted_repo_names() {
    assert_eq!(
        parse_github_remote("https://github.com/socketio/socket.io.git"),
        Some(remote("socketio", "socket.io"))
    );
    assert_eq!(
        parse_github_remote("https://github.com/vuejs/vue.js"),
        Some(remote("vuejs", "vue.js"))
    );
    assert_eq!(
        parse_github_remote("git@github.com:mrdoob/three.js.git"),
        Some(remote("mrdoob", "three.js"))
    );
    assert_eq!(
        parse_github_remote("https://github.com/jashkenas/backbone.git"),
        Some(remote("jashkenas", "backbone"))
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn returns_null_for_non_github_urls() {
    assert_eq!(
        parse_github_remote("https://gitlab.com/owner/repo.git"),
        None
    );
    assert_eq!(parse_github_remote("git@gitlab.com:owner/repo.git"), None);
    assert_eq!(
        parse_github_remote("https://bitbucket.org/owner/repo"),
        None
    );
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn returns_null_for_invalid_urls() {
    assert_eq!(parse_github_remote("not-a-url"), None);
    assert_eq!(parse_github_remote(""), None);
    assert_eq!(parse_github_remote("github.com"), None);
    assert_eq!(parse_github_remote("https://github.com/"), None);
    assert_eq!(parse_github_remote("https://github.com/owner"), None);
}

#[test]
#[ignore = "porting: cli github parseGitHubRemote not implemented"]
fn returns_null_for_extra_path_segments() {
    assert_eq!(
        parse_github_remote("https://github.com/owner/repo/tree/main"),
        None
    );
    assert_eq!(
        parse_github_remote("https://github.com/owner/repo/blob/main/file.ts"),
        None
    );
}
