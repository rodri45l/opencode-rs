//! Port of packages/opencode/test/util/repository.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: parsing GitHub shorthand / host-path / scp remote
//! references, distinguishing local file repositories, typed parse errors,
//! cache identity independent of spelling, and branch-name validation.
//!
//! `repositoryCachePath` is asserted by suffix only: the reference prepends
//! `Global.Path.repos`, whose Rust equivalent is environment-dependent.
#![allow(dead_code)]

// Fast-wave local stubs: `util::repository` is not implemented in this crate yet.
mod repository {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RemoteReference {
        pub host: String,
        pub path: String,
        pub segments: Vec<String>,
        pub owner: String,
        pub repo: String,
        pub remote: String,
        pub label: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LocalReference {
        pub host: String,
        pub protocol: String,
        pub label: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Reference {
        Remote(RemoteReference),
        Local(LocalReference),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RepositoryError {
        InvalidReference,
        UnsupportedLocal,
        InvalidBranch(String),
    }

    impl std::fmt::Display for RepositoryError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RepositoryError::InvalidReference => write!(f, "invalid repository reference"),
                RepositoryError::UnsupportedLocal => {
                    write!(f, "Local file repositories are not supported")
                }
                RepositoryError::InvalidBranch(message) => write!(f, "{message}"),
            }
        }
    }

    pub fn parse_remote(_input: &str) -> Result<RemoteReference, RepositoryError> {
        Err(RepositoryError::InvalidReference)
    }

    pub fn parse_reference(_input: &str) -> Result<Reference, RepositoryError> {
        Err(RepositoryError::InvalidReference)
    }

    pub fn cache_path(_reference: &RemoteReference) -> Result<String, RepositoryError> {
        Err(RepositoryError::InvalidReference)
    }

    pub fn cache_identity(_reference: &RemoteReference) -> Result<String, RepositoryError> {
        Err(RepositoryError::InvalidReference)
    }

    pub fn same_reference(
        _left: &RemoteReference,
        _right: &RemoteReference,
    ) -> Result<bool, RepositoryError> {
        Err(RepositoryError::InvalidReference)
    }

    pub fn is_file_reference(reference: &Reference) -> bool {
        matches!(reference, Reference::Local(_))
    }

    pub fn is_remote_reference(reference: &Reference) -> bool {
        matches!(reference, Reference::Remote(_))
    }

    pub fn validate_branch(_branch: &str) -> Result<(), RepositoryError> {
        Err(RepositoryError::InvalidBranch(
            "porting: repository::validateBranch not implemented".to_string(),
        ))
    }
}

use repository::RepositoryError;

#[test]
#[ignore = "porting: repository not implemented"]
fn parses_github_shorthand_and_preserves_cache_path() {
    let reference = repository::parse_remote("owner/repo").unwrap();

    assert_eq!(reference.host, "github.com");
    assert_eq!(reference.path, "owner/repo");
    assert_eq!(reference.segments, vec!["owner", "repo"]);
    assert_eq!(reference.owner, "owner");
    assert_eq!(reference.repo, "repo");
    assert_eq!(reference.label, "owner/repo");

    assert!(repository::cache_path(&reference)
        .unwrap()
        .ends_with("github.com/owner/repo"));
    assert_eq!(
        repository::cache_identity(&reference).unwrap(),
        "github.com/owner/repo"
    );
}

#[test]
#[ignore = "porting: repository not implemented"]
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
#[ignore = "porting: repository not implemented"]
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
#[ignore = "porting: repository not implemented"]
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
#[ignore = "porting: repository not implemented"]
fn compares_cache_identity_independent_of_input_spelling() {
    let shorthand = repository::parse_remote("owner/repo").unwrap();
    let url = repository::parse_remote("https://github.com/owner/repo.git").unwrap();
    let host_path = repository::parse_remote("github.com/owner/repo").unwrap();

    assert!(repository::same_reference(&shorthand, &url).unwrap());
    assert!(repository::same_reference(&shorthand, &host_path).unwrap());
}

#[test]
#[ignore = "porting: repository not implemented"]
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
