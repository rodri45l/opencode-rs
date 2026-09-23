//! Reference system-context guidance.
//!
//! Ports the observable behaviour of `packages/core/src/reference/guidance.ts`:
//! render an `<available_references>` block for references that carry a
//! description, and omit the block entirely when none qualify.

use crate::path::AbsolutePath;
use crate::CoreResult;

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
    pub fn render(references: &[ReferenceInfo]) -> CoreResult<String> {
        let described: Vec<&ReferenceInfo> = references
            .iter()
            .filter(|reference| {
                reference
                    .description
                    .as_ref()
                    .is_some_and(|description| !description.is_empty())
            })
            .collect();
        if described.is_empty() {
            return Ok(String::new());
        }
        let mut lines = vec!["<available_references>".to_string()];
        for reference in described {
            lines.push("  <reference>".to_string());
            lines.push(format!("    <name>{}</name>", reference.name));
            lines.push(format!("    <path>{}</path>", reference.path));
            lines.push(format!(
                "    <description>{}</description>",
                reference.description.as_deref().unwrap_or_default()
            ));
            lines.push("  </reference>".to_string());
        }
        lines.push("</available_references>".to_string());
        Ok(lines.join("\n"))
    }
}
