//! Filesystem helpers used by tools and config loading.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/filesystem.ts`
//! and `packages/core/src/fs-util.ts`: predicates, JSON round-trip, directory
//! creation, upward search, MIME lookup and path containment.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Whether `path` is an existing directory.
pub fn is_dir(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

/// Whether `path` is an existing regular file.
pub fn is_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

/// Read and parse a JSON file.
pub fn read_json(path: &Path) -> Result<Value, std::io::Error> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(std::io::Error::other)
}

/// Serialize `value` as pretty JSON and write it to `path`.
pub fn write_json(path: &Path, value: &Value) -> Result<(), std::io::Error> {
    let text = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    fs::write(path, text)
}

/// Create `path` and any missing parents.
pub fn ensure_dir(path: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(path)
}

/// Walk up from `start` to `stop`, returning existing `current/target` paths.
pub fn find_up(targets: &[&str], start: &Path, stop: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut current = start.to_path_buf();
    loop {
        for target in targets {
            let search = current.join(target);
            if search.exists() {
                result.push(search);
            }
        }
        if current == stop {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }
    result
}

/// Whether `path` exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Remove a file.
pub fn remove(path: &Path) -> Result<(), std::io::Error> {
    fs::remove_file(path)
}

/// Resolve a MIME type from a file name's extension.
pub fn mime_type(path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mime = match extension.as_str() {
        "json" => "application/json",
        "jsonc" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" | "cjs" => "text/javascript",
        "ts" | "mts" | "cts" => "text/typescript",
        "jsx" => "text/jsx",
        "tsx" => "text/tsx",
        "xml" => "application/xml",
        "yaml" | "yml" => "application/yaml",
        "toml" => "application/toml",
        "csv" => "text/csv",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    };
    mime.to_string()
}

/// Whether `child` is contained within `parent` (or equals it).
pub fn contains(parent: &str, child: &str) -> bool {
    match Path::new(child).strip_prefix(Path::new(parent)) {
        Ok(rest) => {
            rest.as_os_str().is_empty()
                || !rest
                    .components()
                    .next()
                    .is_some_and(|component| component.as_os_str() == "..")
        }
        Err(_) => false,
    }
}

/// Whether two paths contain one another.
pub fn overlaps(left: &str, right: &str) -> bool {
    contains(left, right) || contains(right, left)
}
