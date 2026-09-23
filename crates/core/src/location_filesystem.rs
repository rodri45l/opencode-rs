//! Location-scoped filesystem reads.
//!
//! Ports the observable behaviour of `packages/core/src/filesystem.ts`: read
//! text and binary files (reporting a MIME type), list direct children, and
//! reject lexical escapes out of the active location.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// File contents plus a MIME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContent {
    /// Raw bytes.
    pub content: Vec<u8>,
    /// Detected MIME type.
    pub mime: String,
}

/// A directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// Path relative to the listed directory.
    pub path: String,
    /// `file` or `directory`.
    pub entry_type: String,
}

/// Location-scoped filesystem access.
#[derive(Debug, Default)]
pub struct FileSystem;

impl FileSystem {
    /// Read a file relative to `directory`.
    pub fn read(&self, directory: &AbsolutePath, path: &str) -> CoreResult<FileContent> {
        let canonical = resolve(directory, path)?;
        let content =
            std::fs::read(&canonical).map_err(|error| CoreError::FileSystem(error.to_string()))?;
        let mime = mime_type(path);
        Ok(FileContent { content, mime })
    }

    /// List direct children of `directory`.
    pub fn list(&self, directory: &AbsolutePath) -> CoreResult<Vec<DirEntry>> {
        let entries = std::fs::read_dir(directory.as_path())
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        let mut directories = Vec::new();
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| CoreError::FileSystem(error.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
            if is_dir {
                directories.push(DirEntry {
                    path: format!("{name}{}", std::path::MAIN_SEPARATOR),
                    entry_type: "directory".to_string(),
                });
            } else {
                files.push(DirEntry {
                    path: name,
                    entry_type: "file".to_string(),
                });
            }
        }
        directories.sort_by(|left, right| left.path.cmp(&right.path));
        files.sort_by(|left, right| left.path.cmp(&right.path));
        directories.extend(files);
        Ok(directories)
    }
}

fn resolve(directory: &AbsolutePath, path: &str) -> CoreResult<std::path::PathBuf> {
    let base = directory.as_path();
    let joined = lexical(&base.join(path));
    if !joined.starts_with(base) {
        return Err(CoreError::Invalid(format!(
            "path escapes the active location: {path}"
        )));
    }
    Ok(joined)
}

fn lexical(path: &std::path::Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn mime_type(path: &str) -> String {
    match std::path::Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("txt") | Some("log") => "text/plain",
        Some("md") => "text/markdown",
        Some("json") => "application/json",
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "text/javascript",
        Some("ts") => "application/typescript",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
    .to_string()
}
