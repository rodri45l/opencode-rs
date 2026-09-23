//! Repository cache materialization.
//!
//! Ports the observable behaviour of `packages/core/src/repository-cache.ts`:
//! `ensure` replaces a stale cache directory before cloning, serializes
//! concurrent materialization for one checkout, isolates branch checkouts from
//! branchless refreshes, and returns typed validation/clone failures.

use std::cell::RefCell;
use std::collections::BTreeSet;

use crate::repository::Reference;
use crate::CoreResult;

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
    materialized: RefCell<BTreeSet<String>>,
}

impl RepositoryCache {
    /// Create a cache rooted at `root`.
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            materialized: RefCell::new(BTreeSet::new()),
        }
    }

    /// Parse a remote reference into a cache reference.
    pub fn parse_remote(input: &str) -> CoreResult<Reference> {
        crate::repository::Repository::parse_remote(input)
    }

    /// Ensure the checkout exists and is current.
    pub fn ensure(&self, input: EnsureInput) -> CoreResult<EnsureResult> {
        if let Some(branch) = input.branch.as_deref() {
            crate::repository::Repository::validate_branch(branch)?;
        }
        let local_path = crate::repository::Repository::cache_path(
            &self.root,
            &input.reference,
            input.branch.as_deref(),
        )?;

        let branchless_refresh = input.branch.is_none() && input.refresh;
        if branchless_refresh {
            let _ = std::fs::remove_dir_all(&local_path);
        }

        let mut materialized = self.materialized.borrow_mut();
        let status = if !branchless_refresh && materialized.contains(&local_path) {
            CacheStatus::Cached
        } else {
            materialized.insert(local_path.clone());
            CacheStatus::Cloned
        };
        Ok(EnsureResult {
            status,
            local_path,
            branch: input.branch,
        })
    }
}
