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

/// `parseGitHubRemote`.
fn parse_github_remote(url: &str) -> Option<GitHubRemote> {
    let url = url.strip_prefix("git+").unwrap_or(url);
    let (host, path) = if let Some((_, rest)) = url.split_once("://") {
        let (authority, path) = rest.split_once('/')?;
        let host = authority.rsplit('@').next().unwrap_or(authority);
        (host.to_string(), path.to_string())
    } else if let Some((authority, path)) = url.split_once(':') {
        if !authority.contains('@') {
            return None;
        }
        let host = authority.rsplit('@').next().unwrap_or(authority);
        (host.to_string(), path.to_string())
    } else {
        return None;
    };
    if host != "github.com" {
        return None;
    }
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() != 2 || segments.iter().any(|segment| segment.is_empty()) {
        return None;
    }
    Some(GitHubRemote {
        owner: segments[0].to_string(),
        repo: segments[1].to_string(),
    })
}

#[test]
fn parses_https_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("https://github.com/sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
fn parses_https_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("https://github.com/sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
fn parses_git_at_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("git@github.com:sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
fn parses_git_at_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("git@github.com:sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
fn parses_ssh_url_with_git_suffix() {
    assert_eq!(
        parse_github_remote("ssh://git@github.com/sst/opencode.git"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
fn parses_ssh_url_without_git_suffix() {
    assert_eq!(
        parse_github_remote("ssh://git@github.com/sst/opencode"),
        Some(remote("sst", "opencode"))
    );
}

#[test]
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
fn npm_style_github_shorthand_is_null() {
    assert_eq!(parse_github_remote("github:facebook/react"), None);
}

#[test]
fn parses_http_url() {
    assert_eq!(
        parse_github_remote("http://github.com/owner/repo"),
        Some(remote("owner", "repo"))
    );
}

#[test]
fn parses_hyphenated_names() {
    assert_eq!(
        parse_github_remote("https://github.com/my-org/my-repo.git"),
        Some(remote("my-org", "my-repo"))
    );
}

#[test]
fn parses_underscore_names() {
    assert_eq!(
        parse_github_remote("git@github.com:my_org/my_repo.git"),
        Some(remote("my_org", "my_repo"))
    );
}

#[test]
fn parses_numeric_names() {
    assert_eq!(
        parse_github_remote("https://github.com/org123/repo456"),
        Some(remote("org123", "repo456"))
    );
}

#[test]
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
fn returns_null_for_invalid_urls() {
    assert_eq!(parse_github_remote("not-a-url"), None);
    assert_eq!(parse_github_remote(""), None);
    assert_eq!(parse_github_remote("github.com"), None);
    assert_eq!(parse_github_remote("https://github.com/"), None);
    assert_eq!(parse_github_remote("https://github.com/owner"), None);
}

#[test]
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
