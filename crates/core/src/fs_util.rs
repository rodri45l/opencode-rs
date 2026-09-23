//! Filesystem helpers.
//!
//! Ports the observable behaviour of `packages/core/src/fs-util.ts`: path
//! predicates, safe reads, JSON round-trips, directory creation, upward search,
//! globbing, and the pure `mimeType`/`contains`/`overlaps` helpers.

use std::path::{Path, PathBuf};

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
    pub fn is_dir(_path: &Path) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::is_dir"))
    }

    /// Whether `path` is a file.
    pub fn is_file(_path: &Path) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::is_file"))
    }

    /// Read a file as UTF-8, returning `None` when it does not exist.
    pub fn read_file_string_safe(_path: &Path) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "fs_util::FSUtil::read_file_string_safe",
        ))
    }

    /// Read and decode a JSON file.
    pub fn read_json(_path: &Path) -> CoreResult<Value> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::read_json"))
    }

    /// Encode and write a JSON file.
    pub fn write_json(_path: &Path, _value: &Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::write_json"))
    }

    /// Create a directory and its parents, idempotently.
    pub fn ensure_dir(_path: &Path) -> CoreResult<()> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::ensure_dir"))
    }

    /// Write bytes, creating parent directories as needed.
    pub fn write_with_dirs(_path: &Path, _content: &[u8]) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "fs_util::FSUtil::write_with_dirs",
        ))
    }

    /// Find `target` at or above `start`, stopping at `stop`.
    pub fn find_up(
        _target: &str,
        _start: &Path,
        _stop: Option<&Path>,
    ) -> CoreResult<Vec<AbsolutePath>> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::find_up"))
    }

    /// Collect every `targets` match while walking up from `start`.
    pub fn up(_options: UpOptions) -> CoreResult<Vec<AbsolutePath>> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::up"))
    }

    /// Glob a pattern relative to `cwd`.
    pub fn glob(_pattern: &str, _cwd: &Path, _absolute: bool) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::glob"))
    }

    /// Glob a pattern while walking up from `start` to `stop`.
    pub fn glob_up(_pattern: &str, _start: &Path, _stop: &Path) -> CoreResult<Vec<AbsolutePath>> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::glob_up"))
    }

    /// Match a glob pattern against a path fragment.
    pub fn glob_match(_pattern: &str, _value: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::glob_match"))
    }

    /// The MIME type for a file name.
    pub fn mime_type(_name: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::mime_type"))
    }

    /// Whether `child` is contained by `parent`.
    pub fn contains(_parent: &str, _child: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::contains"))
    }

    /// Whether two paths overlap.
    pub fn overlaps(_a: &str, _b: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_util::FSUtil::overlaps"))
    }
}
