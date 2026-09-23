//! Repository reference parsing and cache paths.
//!
//! Ports the observable behaviour of `packages/core/src/repository.ts`: parse
//! GitHub shorthand, host/path, scp-style and file remotes, derive explicit-root
//! cache paths (with branch suffix encoding), validate branches, and compare
//! cache identity independent of spelling.

use crate::{CoreError, CoreResult};

/// A parsed repository reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// Host (`github.com`, `file`, ...).
    pub host: String,
    /// Path within the host (`owner/repo`).
    pub path: String,
    /// Path segments.
    pub segments: Vec<String>,
    /// Owner segment, when present.
    pub owner: Option<String>,
    /// Repository name.
    pub repo: String,
    /// Clone remote.
    pub remote: String,
    /// Display label.
    pub label: String,
    /// URL protocol, when parsed from a URL.
    pub protocol: Option<String>,
}

/// Repository reference helpers.
#[derive(Debug, Default)]
pub struct Repository;

impl Repository {
    /// Parse an explicit remote URL (`https://`, `ssh://`, `file:`, scp).
    pub fn parse(_url: &str) -> CoreResult<Reference> {
        Err(CoreError::NotImplemented("repository::Repository::parse"))
    }

    /// Parse a remote shorthand (`owner/repo`, `host/group/repo`, scp).
    pub fn parse_remote(_input: &str) -> CoreResult<Reference> {
        Err(CoreError::NotImplemented(
            "repository::Repository::parse_remote",
        ))
    }

    /// Build an explicit-root cache path for a reference and optional branch.
    pub fn cache_path(
        _root: &str,
        _reference: &Reference,
        _branch: Option<&str>,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "repository::Repository::cache_path",
        ))
    }

    /// The stable identity used to compare cache entries.
    pub fn cache_identity(_reference: &Reference) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "repository::Repository::cache_identity",
        ))
    }

    /// Whether the reference is a local file repository.
    pub fn is_file(_reference: &Reference) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("repository::Repository::is_file"))
    }

    /// Whether the reference is a remote repository.
    pub fn is_remote(_reference: &Reference) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "repository::Repository::is_remote",
        ))
    }

    /// Validate a branch name.
    pub fn validate_branch(_branch: &str) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "repository::Repository::validate_branch",
        ))
    }

    /// Whether two references share a cache identity.
    pub fn same(_a: &Reference, _b: &Reference) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("repository::Repository::same"))
    }
}
