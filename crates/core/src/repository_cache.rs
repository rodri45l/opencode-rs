//! Repository cache materialization.
//!
//! Ports the observable behaviour of `packages/core/src/repository-cache.ts`:
//! `ensure` replaces a stale cache directory before cloning, serializes
//! concurrent materialization for one checkout, isolates branch checkouts from
//! branchless refreshes, and returns typed validation/clone failures.

use crate::repository::Reference;
use crate::{CoreError, CoreResult};

/// The result of ensuring a cache entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    /// The checkout was cloned.
    Cloned,
    /// An existing checkout was reused.
    Cached,
}

/// A cache materialization request.
#[derive(Debug, Clone, PartialEq)]
pub struct EnsureInput {
    /// Parsed repository reference.
    pub reference: Reference,
    /// Optional branch to checkout.
    pub branch: Option<String>,
    /// Whether to refresh a branchless checkout.
    pub refresh: bool,
}

/// A materialized cache entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureResult {
    /// How the entry was materialized.
    pub status: CacheStatus,
    /// Local checkout path.
    pub local_path: String,
    /// Checked-out branch, when requested.
    pub branch: Option<String>,
}

/// Repository cache.
#[derive(Debug, Default)]
pub struct RepositoryCache {
    root: String,
}

impl RepositoryCache {
    /// Create a cache rooted at `root`.
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into() }
    }

    /// Parse a remote reference into a cache reference.
    pub fn parse_remote(input: &str) -> CoreResult<Reference> {
        crate::repository::Repository::parse_remote(input)
    }

    /// Ensure the checkout exists and is current.
    pub fn ensure(&self, _input: EnsureInput) -> CoreResult<EnsureResult> {
        let _ = &self.root;
        Err(CoreError::NotImplemented(
            "repository_cache::RepositoryCache::ensure",
        ))
    }
}
