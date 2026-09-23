//! Prompt persistence helpers.
//!
//! Port of packages/tui/src/util/persistence.ts behaviour (upstream 18ef3cc).
//! Re-derived synchronously over `std::fs`.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Read a text file.
pub fn read_text(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path)
}

/// Read and parse a JSON file.
pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<T> {
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Write a text file, creating parent directories.
pub fn write_text(path: impl AsRef<Path>, content: &str) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}

/// Append to a text file, creating parent directories.
pub fn append_text(path: impl AsRef<Path>, content: &str) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(content.as_bytes())
}

/// Atomically write a JSON value via a temporary file and rename.
pub fn write_json_atomic<T: Serialize>(path: impl AsRef<Path>, value: &T) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let serialized = serde_json::to_string(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let write_result = fs::write(&temporary, serialized);
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
        return write_result;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}
