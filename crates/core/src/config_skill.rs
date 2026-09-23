//! Configured skill source resolution.
//!
//! Ports the observable behaviour of `packages/core/src/config/plugin/skill.ts`:
//! every config directory contributes `skill` and `skills` subdirectories, then
//! configured entries are resolved in order — relative paths against the active
//! location, `~/` against the global home, absolute paths unchanged, and URLs
//! passed through as URL sources.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A resolved skill source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSource {
    /// A local directory of skills.
    Directory {
        /// Resolved directory.
        path: AbsolutePath,
    },
    /// A remote skill catalog.
    Url {
        /// Catalog URL.
        url: String,
    },
}

/// Configured skill resolution.
#[derive(Debug, Default)]
pub struct ConfigSkill;

impl ConfigSkill {
    /// Resolve the configured skill sources.
    ///
    /// `config_directories` are the `.opencode` directories from config
    /// entries; `skills` are the raw `skills` array values; `location_directory`
    /// anchors relative entries and `home` expands `~/`.
    pub fn resolve(
        _config_directories: &[AbsolutePath],
        _skills: &[String],
        _location_directory: &AbsolutePath,
        _home: &str,
    ) -> CoreResult<Vec<SkillSource>> {
        Err(CoreError::NotImplemented(
            "config_skill::ConfigSkill::resolve",
        ))
    }
}
