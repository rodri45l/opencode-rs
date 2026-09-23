//! In-memory reference registry.
//!
//! Ports the observable behaviour of `packages/core/src/reference.ts`: scoped
//! registration of normalized local and git sources, git paths derived from the
//! repository cache layout without exposing cache operations, and configured
//! descriptions preserved.

use crate::path::AbsolutePath;
use crate::reference_guidance::{ReferenceInfo, ReferenceSource};
use crate::CoreResult;

/// Reference registry.
#[derive(Debug)]
pub struct Reference {
    repos_root: AbsolutePath,
    entries: Vec<ReferenceInfo>,
}

impl Reference {
    /// Create an empty registry rooted at `repos_root`.
    pub fn new(repos_root: AbsolutePath) -> Self {
        Self {
            repos_root,
            entries: Vec::new(),
        }
    }

    /// Register a reference under `name`.
    pub fn add(&mut self, name: &str, source: ReferenceSource) -> CoreResult<()> {
        let (path, description) = match &source {
            ReferenceSource::Local { path, description } => (path.clone(), description.clone()),
            ReferenceSource::Git {
                repository,
                branch,
                description,
            } => {
                let repository =
                    if repository.contains('/') && !repository.starts_with("github.com") {
                        format!("github.com/{repository}")
                    } else {
                        repository.clone()
                    };
                let base = self.repos_root.as_path().join(repository);
                let path = match branch {
                    Some(branch) if !branch.is_empty() => {
                        AbsolutePath::new(format!("{}@{branch}", base.to_string_lossy()))
                    }
                    _ => AbsolutePath::new(base),
                };
                (path, description.clone())
            }
        };
        self.entries.push(ReferenceInfo {
            name: name.to_string(),
            path,
            description,
            hidden: true,
            source,
        });
        Ok(())
    }

    /// List registered references.
    pub fn list(&self) -> CoreResult<Vec<ReferenceInfo>> {
        Ok(self.entries.clone())
    }

    /// The repos cache root used to derive git reference paths.
    pub fn repos_root(&self) -> CoreResult<AbsolutePath> {
        Ok(self.repos_root.clone())
    }
}
