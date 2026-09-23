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
    pub fn read(&self, _directory: &AbsolutePath, _path: &str) -> CoreResult<FileContent> {
        Err(CoreError::NotImplemented(
            "location_filesystem::FileSystem::read",
        ))
    }

    /// List direct children of `directory`.
    pub fn list(&self, _directory: &AbsolutePath) -> CoreResult<Vec<DirEntry>> {
        Err(CoreError::NotImplemented(
            "location_filesystem::FileSystem::list",
        ))
    }
}
