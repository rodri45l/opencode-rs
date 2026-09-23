//! Port of packages/function/test/github.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/function/src/github.ts; see docs/TEST-PORT.md.

use opencode_function::{parse_repository_claim, ClaimError, RepositoryClaim, RepositoryIdentity};

fn claim(repository: Option<&str>, sub: Option<&str>) -> RepositoryClaim {
    RepositoryClaim {
        repository: repository.map(str::to_string),
        sub: sub.map(str::to_string),
    }
}

#[test]
fn reads_repository_identity_with_a_legacy_subject() {
    let parsed = parse_repository_claim(&claim(
        Some("octocat/my-repo"),
        Some("repo:octocat/my-repo:ref:refs/heads/main"),
    ))
    .unwrap();
    assert_eq!(
        parsed,
        RepositoryIdentity {
            owner: "octocat".into(),
            repo: "my-repo".into(),
        }
    );
}

#[test]
fn reads_repository_identity_with_an_immutable_subject() {
    let parsed = parse_repository_claim(&claim(
        Some("octocat/my-repo"),
        Some("repo:octocat@123456/my-repo@456789:ref:refs/heads/main"),
    ))
    .unwrap();
    assert_eq!(
        parsed,
        RepositoryIdentity {
            owner: "octocat".into(),
            repo: "my-repo".into(),
        }
    );
}

#[test]
fn does_not_depend_on_a_repository_path_in_a_customized_subject() {
    let parsed = parse_repository_claim(&claim(
        Some("octocat/my-repo"),
        Some("repository_owner:octocat:repository_visibility:private"),
    ))
    .unwrap();
    assert_eq!(
        parsed,
        RepositoryIdentity {
            owner: "octocat".into(),
            repo: "my-repo".into(),
        }
    );
}

#[test]
fn rejects_a_missing_repository_claim() {
    let error = parse_repository_claim(&claim(None, None)).unwrap_err();
    assert_eq!(error, ClaimError::Missing);
    assert_eq!(error.to_string(), "Repository claim is missing");
}

#[test]
fn rejects_an_invalid_repository_claim() {
    let error = parse_repository_claim(&claim(Some("octocat"), None)).unwrap_err();
    assert_eq!(error, ClaimError::Invalid);
    assert_eq!(error.to_string(), "Repository claim is invalid");
}
