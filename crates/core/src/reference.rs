//! In-memory reference registry.
//!
//! Ports the observable behaviour of `packages/core/src/reference.ts`: scoped
//! registration of normalized local and git sources, git paths derived from the
//! repository cache layout without exposing cache operations, and configured
//! descriptions preserved.

use crate::path::AbsolutePath;
use crate::reference_guidance::{ReferenceInfo, ReferenceSource};
use crate::{CoreError, CoreResult};

/// Reference registry.
#[derive(Debug)]
pub struct Reference {
    repos_root: AbsolutePath,
}

impl Reference {
    /// Create an empty registry rooted at `repos_root`.
    pub fn new(repos_root: AbsolutePath) -> Self {
        Self { repos_root }
    }

    /// Register a reference under `name`.
    pub fn add(&mut self, _name: &str, _source: ReferenceSource) -> CoreResult<()> {
        let _ = &self.repos_root;
        Err(CoreError::NotImplemented("reference::Reference::add"))
    }

    /// List registered references.
    pub fn list(&self) -> CoreResult<Vec<ReferenceInfo>> {
        Err(CoreError::NotImplemented("reference::Reference::list"))
    }

    /// The repos cache root used to derive git reference paths.
    pub fn repos_root(&self) -> CoreResult<AbsolutePath> {
        Err(CoreError::NotImplemented(
            "reference::Reference::repos_root",
        ))
    }
}
