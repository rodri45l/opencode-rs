//! Reference system-context guidance.
//!
//! Ports the observable behaviour of `packages/core/src/reference/guidance.ts`:
//! render an `<available_references>` block for references that carry a
//! description, and omit the block entirely when none qualify.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A reference source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceSource {
    /// A local path.
    Local {
        /// Source path.
        path: AbsolutePath,
        /// Optional description.
        description: Option<String>,
    },
    /// A git repository.
    Git {
        /// Repository reference.
        repository: String,
        /// Optional branch.
        branch: Option<String>,
        /// Optional description.
        description: Option<String>,
    },
}

/// A resolved reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceInfo {
    /// Reference name.
    pub name: String,
    /// Resolved path.
    pub path: AbsolutePath,
    /// Optional description.
    pub description: Option<String>,
    /// Whether the reference is hidden from the model.
    pub hidden: bool,
    /// The originating source.
    pub source: ReferenceSource,
}

/// Reference guidance rendering.
#[derive(Debug, Default)]
pub struct ReferenceGuidance;

impl ReferenceGuidance {
    /// Render the available-references system context block.
    pub fn render(_references: &[ReferenceInfo]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "reference_guidance::ReferenceGuidance::render",
        ))
    }
}
