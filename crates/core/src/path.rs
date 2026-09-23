//! Canonical absolute paths.
//!
//! The reference implementation models every filesystem root as an
//! `AbsolutePath`; the Rust port keeps a thin newtype over [`PathBuf`] so
//! modules agree on one path vocabulary.

use std::fmt;
use std::path::{Path, PathBuf};

/// An absolute filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AbsolutePath(PathBuf);

impl AbsolutePath {
    /// Construct from a path-like value.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Borrow the path.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl From<&Path> for AbsolutePath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<PathBuf> for AbsolutePath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl AsRef<Path> for AbsolutePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}
