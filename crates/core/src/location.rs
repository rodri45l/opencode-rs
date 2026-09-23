//! Resolved working location.
//!
//! Ports `packages/core/src/location.ts`: a location binds a workspace to a
//! directory and resolves the enclosing project plus its VCS metadata.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};
use opencode_schema::WorkspaceId;

/// Identifier for a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectId(String);

impl ProjectId {
    /// Construct a project id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Version-control metadata for a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vcs {
    /// A git checkout, storing objects under `store`.
    Git {
        /// The git directory.
        store: AbsolutePath,
    },
}

/// A resolved project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInfo {
    /// Project id.
    pub id: ProjectId,
    /// Project root directory.
    pub directory: AbsolutePath,
    /// Version control, when detected.
    pub vcs: Option<Vcs>,
}

/// A resolved location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationInfo {
    /// The active directory.
    pub directory: AbsolutePath,
    /// The active workspace.
    pub workspace_id: WorkspaceId,
    /// The enclosing project.
    pub project: ProjectInfo,
    /// Version control, when detected.
    pub vcs: Option<Vcs>,
}

/// Resolves the current location's project and VCS.
#[derive(Debug, Default)]
pub struct Location;

impl Location {
    /// Resolve a bound location.
    pub fn resolve(
        _directory: &AbsolutePath,
        _workspace_id: WorkspaceId,
    ) -> CoreResult<LocationInfo> {
        Err(CoreError::NotImplemented("location::Location::resolve"))
    }
}
