//! Cassette file discovery and path validation.
//!
//! Derived from the observable behaviour pinned by
//! `packages/http-recorder/src/cassette.ts` (upstream 18ef3cc).

use std::path::{Path, PathBuf};

/// Errors raised while resolving cassette paths.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CassetteError {
    #[error("Invalid cassette name")]
    InvalidName,
}

fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.contains('\\')
        && !name.contains(':')
        && !name.starts_with('/')
}

/// Whether a named cassette exists inside the recordings directory.
pub fn has_cassette_sync(name: &str, directory: &Path) -> Result<bool, CassetteError> {
    if !is_safe_name(name) {
        return Err(CassetteError::InvalidName);
    }
    Ok(directory.join(format!("{name}.json")).is_file())
}

fn collect(directory: &Path, prefix: &str, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            let next = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            collect(&path, &next, out);
        } else if let Some(stem) = name.strip_suffix(".json") {
            out.push(if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}/{stem}")
            });
        }
    }
}

/// Enumerate recorded cassette names, relative to the recordings directory.
pub fn list_cassettes(directory: &Path) -> Vec<String> {
    let mut names = Vec::new();
    collect(directory, "", &mut names);
    names.sort();
    names
}

/// The absolute path of a named cassette inside the recordings directory.
pub fn cassette_path(name: &str, directory: &Path) -> Result<PathBuf, CassetteError> {
    if !is_safe_name(name) {
        return Err(CassetteError::InvalidName);
    }
    Ok(directory.join(format!("{name}.json")))
}
