//! Serverless function runtime surface.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.

/// Minimal GitHub OIDC claim shape used to resolve the repository identity.
///
/// The reference `parseRepositoryClaim` only reads the `repository` claim; the
/// `sub` claim is accepted for shape compatibility but does not affect identity.
#[derive(Debug, Clone, Default)]
pub struct RepositoryClaim {
    pub repository: Option<String>,
    pub sub: Option<String>,
}

/// The `owner`/`repo` pair extracted from a GitHub repository claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIdentity {
    pub owner: String,
    pub repo: String,
}

/// Errors produced while parsing a repository claim.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClaimError {
    #[error("Repository claim is missing")]
    Missing,
    #[error("Repository claim is invalid")]
    Invalid,
}

/// Parse a GitHub repository identity from an OIDC claim.
///
/// The claim is only valid when `repository` is a `owner/repo` pair with both
/// halves present; the `sub` claim (legacy, immutable, or customised) is not
/// consulted for the identity.
pub fn parse_repository_claim(claim: &RepositoryClaim) -> Result<RepositoryIdentity, ClaimError> {
    let repository = claim.repository.as_deref().unwrap_or("").trim();
    if repository.is_empty() {
        return Err(ClaimError::Missing);
    }
    let mut parts = repository.split('/');
    let owner = parts.next().unwrap_or("");
    let repo = parts.next().unwrap_or("");
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return Err(ClaimError::Invalid);
    }
    Ok(RepositoryIdentity {
        owner: owner.to_string(),
        repo: repo.to_string(),
    })
}
