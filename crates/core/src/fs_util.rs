//! Filesystem helpers.
//!
//! Ports the observable behaviour of `packages/core/src/fs-util.ts`: path
//! predicates, safe reads, JSON round-trips, directory creation, upward search,
//! globbing, and the pure `mimeType`/`contains`/`overlaps` helpers.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// Options for [`FSUtil::up`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpOptions {
    /// File names to look for while walking up.
    pub targets: Vec<String>,
    /// Directory to start from.
    pub start: PathBuf,
    /// Directory to stop at (inclusive).
    pub stop: Option<PathBuf>,
}

/// Filesystem helpers.
#[derive(Debug, Default)]
pub struct FSUtil;

impl FSUtil {
    /// Whether `path` is a directory.
    pub fn is_dir(path: &Path) -> CoreResult<bool> {
        Ok(path.is_dir())
    }

    /// Whether `path` is a file.
    pub fn is_file(path: &Path) -> CoreResult<bool> {
        Ok(path.is_file())
    }

    /// Read a file as UTF-8, returning `None` when it does not exist.
    pub fn read_file_string_safe(path: &Path) -> CoreResult<Option<String>> {
        match std::fs::read(path) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|error| CoreError::FileSystem(error.to_string())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CoreError::FileSystem(error.to_string())),
        }
    }

    /// Read and decode a JSON file.
    pub fn read_json(path: &Path) -> CoreResult<Value> {
        let bytes =
            std::fs::read(path).map_err(|error| CoreError::FileSystem(error.to_string()))?;
        serde_json::from_slice(&bytes).map_err(|error| CoreError::FileSystem(error.to_string()))
    }

    /// Encode and write a JSON file.
    pub fn write_json(path: &Path, value: &Value) -> CoreResult<()> {
        let encoded = serde_json::to_vec_pretty(value)
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        Self::write_with_dirs(path, &encoded)
    }

    /// Create a directory and its parents, idempotently.
    pub fn ensure_dir(path: &Path) -> CoreResult<()> {
        std::fs::create_dir_all(path).map_err(|error| CoreError::FileSystem(error.to_string()))
    }

    /// Write bytes, creating parent directories as needed.
    pub fn write_with_dirs(path: &Path, content: &[u8]) -> CoreResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                Self::ensure_dir(parent)?;
            }
        }
        std::fs::write(path, content).map_err(|error| CoreError::FileSystem(error.to_string()))
    }

    /// Find `target` at or above `start`, stopping at `stop`.
    pub fn find_up(
        target: &str,
        start: &Path,
        stop: Option<&Path>,
    ) -> CoreResult<Vec<AbsolutePath>> {
        let mut found = Vec::new();
        let mut current = Some(start);
        while let Some(directory) = current {
            let candidate = directory.join(target);
            if candidate.exists() {
                found.push(AbsolutePath::new(candidate));
            }
            if Some(directory) == stop {
                break;
            }
            current = directory.parent();
        }
        Ok(found)
    }

    /// Collect every `targets` match while walking up from `start`.
    pub fn up(options: UpOptions) -> CoreResult<Vec<AbsolutePath>> {
        let stop = options.stop.as_deref();
        let mut found = Vec::new();
        let mut current = Some(options.start.as_path());
        while let Some(directory) = current {
            for target in &options.targets {
                let candidate = directory.join(target);
                if candidate.exists() {
                    found.push(AbsolutePath::new(candidate));
                }
            }
            if Some(directory) == stop {
                break;
            }
            current = directory.parent();
        }
        Ok(found)
    }

    /// Glob a pattern relative to `cwd`.
    pub fn glob(pattern: &str, cwd: &Path, absolute: bool) -> CoreResult<Vec<String>> {
        if pattern.contains('/') {
            return Self::glob_walk(pattern, cwd, absolute);
        }
        let mut matches = Vec::new();
        let entries =
            std::fs::read_dir(cwd).map_err(|error| CoreError::FileSystem(error.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|error| CoreError::FileSystem(error.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if Self::glob_match(pattern, &name)? {
                let value = if absolute {
                    entry.path().to_string_lossy().to_string()
                } else {
                    name
                };
                matches.push(value);
            }
        }
        Ok(matches)
    }

    fn glob_walk(pattern: &str, cwd: &Path, absolute: bool) -> CoreResult<Vec<String>> {
        let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let mut results = Vec::new();
        Self::glob_segments(&segments, cwd, cwd, absolute, &mut results)?;
        Ok(results)
    }

    fn glob_segments(
        segments: &[&str],
        base: &Path,
        current: &Path,
        absolute: bool,
        out: &mut Vec<String>,
    ) -> CoreResult<()> {
        if segments.is_empty() {
            return Ok(());
        }
        let (head, tail) = segments.split_first().expect("non-empty");
        if *head == "**" {
            Self::glob_segments(tail, base, current, absolute, out)?;
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        Self::glob_segments(segments, base, &entry.path(), absolute, out)?;
                    }
                }
            }
            return Ok(());
        }
        let entries = match std::fs::read_dir(current) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !Self::glob_match(head, &name)? {
                continue;
            }
            let path = entry.path();
            if tail.is_empty() {
                if absolute {
                    out.push(path.to_string_lossy().to_string());
                } else {
                    out.push(
                        path.strip_prefix(base)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string(),
                    );
                }
            } else if path.is_dir() {
                Self::glob_segments(tail, base, &path, absolute, out)?;
            }
        }
        Ok(())
    }

    /// Glob a pattern while walking up from `start` to `stop`.
    pub fn glob_up(pattern: &str, start: &Path, stop: &Path) -> CoreResult<Vec<AbsolutePath>> {
        let mut found = Vec::new();
        let mut current = Some(start);
        while let Some(directory) = current {
            for matched in Self::glob(pattern, directory, true)? {
                found.push(AbsolutePath::new(matched));
            }
            if directory == stop {
                break;
            }
            current = directory.parent();
        }
        Ok(found)
    }

    /// Match a glob pattern against a path fragment.
    pub fn glob_match(pattern: &str, value: &str) -> CoreResult<bool> {
        Ok(glob_match(pattern, value))
    }

    /// The MIME type for a file name.
    pub fn mime_type(name: &str) -> CoreResult<String> {
        let extension = Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mime = match extension.as_str() {
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "md" => "text/markdown",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" | "mjs" | "cjs" => "text/javascript",
            "ts" => "application/typescript",
            "wasm" => "application/wasm",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        };
        Ok(mime.to_string())
    }

    /// Whether `child` is contained by `parent`.
    pub fn contains(parent: &str, child: &str) -> CoreResult<bool> {
        if parent == child {
            return Ok(true);
        }
        let parent = normalized_components(parent);
        let child = normalized_components(child);
        Ok(child.len() > parent.len() && child[..parent.len()] == parent[..])
    }

    /// Whether two paths overlap.
    pub fn overlaps(a: &str, b: &str) -> CoreResult<bool> {
        Ok(Self::contains(a, b)? || Self::contains(b, a)?)
    }
}

fn normalized_components(path: &str) -> Vec<String> {
    Path::new(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_string_lossy().to_string()),
            Component::RootDir => Some("/".to_string()),
            Component::CurDir => None,
            _ => None,
        })
        .collect()
}

fn glob_match(pattern: &str, value: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let value: Vec<char> = value.chars().collect();
    match_pattern(&pattern, &value)
}

fn match_pattern(pattern: &[char], value: &[char]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }
    match pattern[0] {
        '*' => {
            if pattern.get(1) == Some(&'*') {
                let rest = &pattern[2..];
                if rest.first() == Some(&'/') {
                    let rest = &rest[1..];
                    if match_pattern(rest, value) {
                        return true;
                    }
                    for index in 0..value.len() {
                        if value[index] == '/' && match_pattern(rest, &value[index + 1..]) {
                            return true;
                        }
                    }
                    return false;
                }
                for index in 0..=value.len() {
                    if match_pattern(rest, &value[index..]) {
                        return true;
                    }
                }
                return false;
            }
            let rest = &pattern[1..];
            for index in 0..=value.len() {
                if match_pattern(rest, &value[index..]) {
                    return true;
                }
                if index < value.len() && value[index] == '/' {
                    break;
                }
            }
            false
        }
        '?' => !value.is_empty() && value[0] != '/' && match_pattern(&pattern[1..], &value[1..]),
        character => {
            !value.is_empty() && value[0] == character && match_pattern(&pattern[1..], &value[1..])
        }
    }
}
