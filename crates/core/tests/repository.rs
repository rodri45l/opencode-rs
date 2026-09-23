//! Port of packages/core/test/repository.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: parse GitHub shorthand, host/path and scp remotes, derive
//! explicit-root cache paths (encoding branches), reject unsafe references and
//! branches with typed errors, keep local file repositories distinct, and
//! compare cache identity independent of input spelling.

use std::path::Path;

use opencode_core::repository::Repository;

const NOTE: &str = "porting: repository not implemented";

#[test]
#[ignore = "porting: repository not implemented"]
fn parses_github_shorthand_and_builds_an_explicit_root_cache_path() {
    let reference = Repository::parse_remote("owner/repo").expect(NOTE);

    assert_eq!(reference.host, "github.com");
    assert_eq!(reference.path, "owner/repo");
    assert_eq!(reference.segments, vec!["owner", "repo"]);
    assert_eq!(reference.owner.as_deref(), Some("owner"));
    assert_eq!(reference.repo, "repo");
    assert_eq!(reference.remote, "https://github.com/owner/repo.git");
    assert_eq!(reference.label, "owner/repo");

    assert_eq!(
        Repository::cache_path("/cache", &reference, None).expect(NOTE),
        Path::new("/cache")
            .join("github.com")
            .join("owner")
            .join("repo")
            .to_string_lossy()
            .to_string()
    );
    assert_eq!(
        Repository::cache_path("/cache", &reference, Some("main")).expect(NOTE),
        Path::new("/cache")
            .join("github.com")
            .join("owner")
            .join("repo@main")
            .to_string_lossy()
            .to_string()
    );
    assert_eq!(
        Repository::cache_path("/cache", &reference, Some("feature/x")).expect(NOTE),
        Path::new("/cache")
            .join("github.com")
            .join("owner")
            .join("repo@feature%2Fx")
            .to_string_lossy()
            .to_string()
    );
    assert_eq!(
        Repository::cache_identity(&reference).expect(NOTE),
        "github.com/owner/repo"
    );
}

#[test]
#[ignore = "porting: repository not implemented"]
fn parses_host_path_and_scp_remote_references() {
    let gitlab = Repository::parse_remote("gitlab.com/group/repo").expect(NOTE);
    assert_eq!(gitlab.host, "gitlab.com");
    assert_eq!(gitlab.path, "group/repo");
    assert_eq!(gitlab.remote, "https://gitlab.com/group/repo.git");
    assert_eq!(gitlab.label, "gitlab.com/group/repo");

    let scp = Repository::parse_remote("git@github.com:owner/repo.git").expect(NOTE);
    assert_eq!(scp.host, "github.com");
    assert_eq!(scp.path, "owner/repo");
    assert_eq!(scp.remote, "git@github.com:owner/repo.git");
    assert_eq!(scp.label, "owner/repo");
}

#[test]
#[ignore = "porting: repository not implemented"]
fn keeps_local_file_repositories_distinct_from_remote_repositories() {
    let local_path = std::path::Path::new("repo.git");
    let url = format!("file://{}", local_path.to_string_lossy());
    let reference = Repository::parse(&url).expect(NOTE);

    assert_eq!(reference.host, "file");
    assert_eq!(reference.protocol.as_deref(), Some("file:"));
    assert!(Repository::is_file(&reference).expect(NOTE));
    assert!(!Repository::is_remote(&reference).expect(NOTE));
    assert!(Repository::parse_remote(&url).is_err());
}

#[test]
#[ignore = "porting: repository not implemented"]
fn rejects_unsafe_remote_references_and_branches_with_typed_errors() {
    assert!(Repository::parse_remote("not-a-repo").is_err());
    assert!(Repository::parse_remote("git@github.com:../../../etc/passwd").is_err());
    assert!(Repository::validate_branch("feature/docs.v1").is_ok());
    assert!(Repository::validate_branch("-bad").is_err());
    assert!(Repository::validate_branch("bad..branch").is_err());
    assert!(Repository::validate_branch("bad branch").is_err());
}

#[test]
#[ignore = "porting: repository not implemented"]
fn compares_cache_identity_independent_of_input_spelling() {
    let shorthand = Repository::parse_remote("owner/repo").expect(NOTE);

    assert!(Repository::same(
        &shorthand,
        &Repository::parse_remote("https://github.com/owner/repo.git").expect(NOTE)
    )
    .expect(NOTE));
    assert!(Repository::same(
        &shorthand,
        &Repository::parse_remote("github.com/owner/repo").expect(NOTE)
    )
    .expect(NOTE));
}
