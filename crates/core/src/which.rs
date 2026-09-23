//! `which` command lookup.
//!
//! Ports the observable behaviour of `packages/core/src/util/which.ts`: locate an
//! executable on a PATH (honoring PATHEXT on Windows), returning the first match
//! or `None`.

use std::path::PathBuf;

use crate::{CoreError, CoreResult};

/// Command lookup.
#[derive(Debug, Default)]
pub struct Which;

impl Which {
    /// Locate `command` on `path`, optionally using `pathext`.
    pub fn which(command: &str, path: &str, pathext: Option<&str>) -> CoreResult<Option<PathBuf>> {
        let extensions: Vec<String> = if cfg!(windows) {
            pathext
                .map(|value| {
                    value
                        .split(';')
                        .map(|part| part.trim().to_string())
                        .filter(|part| !part.is_empty())
                        .collect()
                })
                .unwrap_or_else(|| vec![".COM".into(), ".EXE".into(), ".BAT".into(), ".CMD".into()])
        } else {
            Vec::new()
        };

        let candidates: Vec<String> =
            if cfg!(windows) && PathBuf::from(command).extension().is_none() {
                let mut list = vec![command.to_string()];
                list.extend(
                    extensions
                        .iter()
                        .map(|extension| format!("{command}{extension}")),
                );
                list
            } else {
                vec![command.to_string()]
            };

        for dir in std::env::split_paths(path) {
            for candidate in &candidates {
                if let Some(found) = Self::check(&dir, candidate) {
                    return Ok(Some(found));
                }
            }
        }
        Ok(None)
    }

    fn check(dir: &std::path::Path, name: &str) -> Option<PathBuf> {
        let candidate = dir.join(name);
        if !candidate.is_file() {
            return None;
        }
        if !Self::is_executable(&candidate) {
            return None;
        }
        Some(candidate)
    }

    #[cfg(unix)]
    fn is_executable(path: &std::path::Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(metadata) => metadata.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }

    #[cfg(not(unix))]
    fn is_executable(_path: &std::path::Path) -> bool {
        true
    }
}

/// Error helper kept so callers can map a failed lookup.
pub fn not_found(command: &str) -> CoreError {
    CoreError::Message(format!("command not found: {command}"))
}
