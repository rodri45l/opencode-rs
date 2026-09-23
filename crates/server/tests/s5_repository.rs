//! Port of packages/opencode/test/util/repository.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: parsing GitHub shorthand / host-path / scp remote
//! references, distinguishing local file repositories, typed parse errors,
//! cache identity independent of spelling, and branch-name validation.
//!
//! `repositoryCachePath` is asserted by suffix only: the reference prepends
//! `Global.Path.repos`, whose Rust equivalent is environment-dependent.
#![allow(dead_code)]

use opencode_server::repository::{self, RepositoryError};

#[test]
fn parses_github_shorthand_and_preserves_cache_path() {
    let reference = repository::parse_remote("owner/repo").unwrap();

    assert_eq!(reference.host, "github.com");
    assert_eq!(reference.path, "owner/repo");
    assert_eq!(reference.segments, vec!["owner", "repo"]);
    assert_eq!(reference.owner, "owner");
    assert_eq!(reference.repo, "repo");
    assert_eq!(reference.label, "owner/repo");

    assert!(repository::cache_path(&reference).ends_with("github.com/owner/repo"));
    assert_eq!(
        repository::cache_identity(&reference),
        "github.com/owner/repo"
    );
}

#[test]
fn parses_host_path_and_scp_remote_references() {
    let host_path = repository::parse_remote("gitlab.com/group/repo").unwrap();
    let scp = repository::parse_remote("git@github.com:owner/repo.git").unwrap();

    assert_eq!(host_path.host, "gitlab.com");
    assert_eq!(host_path.path, "group/repo");
    assert_eq!(host_path.remote, "https://gitlab.com/group/repo.git");
    assert_eq!(host_path.label, "gitlab.com/group/repo");

    assert_eq!(scp.host, "github.com");
    assert_eq!(scp.path, "owner/repo");
    assert_eq!(scp.remote, "git@github.com:owner/repo.git");
    assert_eq!(scp.label, "owner/repo");
}

#[test]
fn keeps_local_file_repositories_distinct_from_remote_repositories() {
    let local_path = "/tmp/oc-repository/repo.git";
    let local = format!("file://{local_path}");
    let reference = repository::parse_reference(&local).unwrap();

    assert_eq!(
        reference,
        repository::Reference::Local(repository::LocalReference {
            host: "file".to_string(),
            protocol: "file:".to_string(),
            label: local_path.to_string(),
        })
    );
    assert!(repository::is_file_reference(&reference));
    assert!(!repository::is_remote_reference(&reference));

    let err = repository::parse_remote(&local).unwrap_err();
    assert!(err
        .to_string()
        .contains("Local file repositories are not supported"));
    assert_eq!(err, RepositoryError::UnsupportedLocal);
}

#[test]
fn rejects_invalid_remote_repository_references_with_typed_errors() {
    assert_eq!(
        repository::parse_remote("not-a-repo").unwrap_err(),
        RepositoryError::InvalidReference
    );
    assert_eq!(
        repository::parse_remote("git@github.com:../../../etc/passwd").unwrap_err(),
        RepositoryError::InvalidReference
    );
}

#[test]
fn compares_cache_identity_independent_of_input_spelling() {
    let shorthand = repository::parse_remote("owner/repo").unwrap();
    let url = repository::parse_remote("https://github.com/owner/repo.git").unwrap();
    let host_path = repository::parse_remote("github.com/owner/repo").unwrap();

    assert!(repository::same_reference(&shorthand, &url));
    assert!(repository::same_reference(&shorthand, &host_path));
}

#[test]
fn validates_repository_branch_names() {
    assert!(repository::validate_branch("feature/docs.v1").is_ok());

    let err = repository::validate_branch("-bad").unwrap_err();
    assert!(err
        .to_string()
        .contains("Branch must contain only alphanumeric characters"));
    assert!(repository::validate_branch("bad..branch")
        .unwrap_err()
        .to_string()
        .contains("Branch must contain only alphanumeric characters"));
    assert!(repository::validate_branch("bad branch")
        .unwrap_err()
        .to_string()
        .contains("Branch must contain only alphanumeric characters"));
    assert!(matches!(
        repository::validate_branch("bad branch").unwrap_err(),
        RepositoryError::InvalidBranch(_)
    ));
}
