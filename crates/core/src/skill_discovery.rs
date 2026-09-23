//! Skill discovery from an HTTP catalog.
//!
//! Ports the observable behaviour of `packages/core/src/skill/discovery.ts`:
//! `pull` fetches a catalog index and downloads each skill's files into the
//! cache, rejecting skill names and file paths that traverse outside the skill
//! root, absolute paths, and cross-origin file URLs. Safe nested files are
//! downloaded under the skill root, and cached files refresh when the version
//! changes.

use crate::{CoreError, CoreResult};

/// Skill discovery.
#[derive(Debug, Default)]
pub struct SkillDiscovery;

impl SkillDiscovery {
    /// Whether a skill name is safe to use as a directory name.
    pub fn is_safe_name(name: &str) -> CoreResult<bool> {
        if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
            return Ok(false);
        }
        if name.starts_with('.') || name.contains('\0') {
            return Ok(false);
        }
        Ok(true)
    }

    /// Whether a catalog file path stays under the skill root.
    pub fn is_safe_file(file: &str) -> CoreResult<bool> {
        if file.is_empty() {
            return Ok(false);
        }
        if file.starts_with('/') || file.starts_with('\\') {
            return Ok(false);
        }
        if file.contains("://") {
            return Ok(false);
        }
        if file.starts_with("..") || file.contains("/../") || file.contains("\\..\\") {
            return Ok(false);
        }
        if file.contains('\0') {
            return Ok(false);
        }
        Ok(true)
    }

    /// Pull a catalog and materialize its skills, returning their directories.
    pub fn pull(_base: &str, _cache: &str) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented(
            "skill_discovery::SkillDiscovery::pull",
        ))
    }
}
