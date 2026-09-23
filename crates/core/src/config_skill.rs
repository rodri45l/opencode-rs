//! Configured skill source resolution.
//!
//! Ports the observable behaviour of `packages/core/src/config/plugin/skill.ts`:
//! every config directory contributes `skill` and `skills` subdirectories, then
//! configured entries are resolved in order — relative paths against the active
//! location, `~/` against the global home, absolute paths unchanged, and URLs
//! passed through as URL sources.

use crate::path::AbsolutePath;
use crate::CoreResult;

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
        config_directories: &[AbsolutePath],
        skills: &[String],
        location_directory: &AbsolutePath,
        home: &str,
    ) -> CoreResult<Vec<SkillSource>> {
        let mut sources = Vec::new();
        for directory in config_directories {
            sources.push(SkillSource::Directory {
                path: AbsolutePath::new(directory.as_path().join("skill")),
            });
            sources.push(SkillSource::Directory {
                path: AbsolutePath::new(directory.as_path().join("skills")),
            });
        }
        let base = location_directory.as_path();
        for entry in skills {
            if entry.starts_with("http://") || entry.starts_with("https://") {
                sources.push(SkillSource::Url { url: entry.clone() });
                continue;
            }
            let path = if let Some(rest) = entry.strip_prefix("~/") {
                AbsolutePath::new(std::path::Path::new(home).join(rest))
            } else if std::path::Path::new(entry).is_absolute() {
                AbsolutePath::new(entry)
            } else {
                AbsolutePath::new(base.join(entry))
            };
            sources.push(SkillSource::Directory { path });
        }
        Ok(sources)
    }
}
