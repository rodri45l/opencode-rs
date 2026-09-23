//! Read-tool filesystem helpers (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/tool/read-filesystem.ts`: reads fail with a typed
//! filesystem error when a resolved file disappears or a listing fails, with a
//! path-kind error when the target is the wrong kind, with binary/malformed-UTF8
//! errors for undecodable content, with an offset error for out-of-range
//! pagination, and with a media-ingest-limit error for oversized media. Reading
//! stops after the requested page, reporting `truncated` and the next offset. The
//! Effect `FSUtil`/`FileSystem` service wiring is replaced by `std::fs`.

use std::fmt;
use std::path::Path;

/// Maximum bytes accepted for media ingestion.
pub const MAX_MEDIA_INGEST_BYTES: usize = 10 * 1024 * 1024;

/// A typed read failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadToolError {
    /// The target had the wrong path kind (for example a directory).
    PathKind(String),
    /// The target is a binary file.
    Binary(String),
    /// The target is not valid UTF-8.
    MalformedUtf8(String),
    /// The requested offset is out of range.
    OffsetOutOfRange(String),
    /// The media target exceeds the ingestion limit.
    MediaIngestLimit(String),
    /// An underlying filesystem operation failed.
    FileSystem {
        /// The filesystem method that failed.
        method: String,
        /// The underlying message.
        message: String,
    },
}

impl fmt::Display for ReadToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathKind(message)
            | Self::Binary(message)
            | Self::MalformedUtf8(message)
            | Self::OffsetOutOfRange(message)
            | Self::MediaIngestLimit(message) => f.write_str(message),
            Self::FileSystem { method, message } => write!(f, "{method}: {message}"),
        }
    }
}

impl std::error::Error for ReadToolError {}

/// A paginated text read result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPage {
    /// The page content.
    pub content: String,
    /// Whether more content follows.
    pub truncated: bool,
    /// The next offset when truncated.
    pub next: Option<usize>,
}

/// The read-tool filesystem helpers.
#[derive(Debug, Default)]
pub struct ReadToolFileSystem;

impl ReadToolFileSystem {
    /// Read `path`, optionally paginated by `offset` (1-based line) and `limit`.
    pub fn read(
        path: &str,
        resource: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<TextPage, ReadToolError> {
        let path_ref = Path::new(path);
        let metadata = std::fs::metadata(path_ref).map_err(|error| ReadToolError::FileSystem {
            method: "readFile".into(),
            message: error.to_string(),
        })?;
        if metadata.is_dir() {
            return Err(ReadToolError::PathKind(format!(
                "Cannot read directory: {resource}"
            )));
        }

        let mime = crate::fs_util::FSUtil::mime_type(resource).unwrap_or_default();
        if mime.starts_with("image/") && metadata.len() > MAX_MEDIA_INGEST_BYTES as u64 {
            return Err(ReadToolError::MediaIngestLimit(format!(
                "Media exceeds {MAX_MEDIA_INGEST_BYTES} byte ingestion limit: {resource}"
            )));
        }

        let bytes = std::fs::read(path_ref).map_err(|error| ReadToolError::FileSystem {
            method: "readFile".into(),
            message: error.to_string(),
        })?;
        if bytes.contains(&0) {
            return Err(ReadToolError::Binary(format!(
                "Cannot read binary file: {resource}"
            )));
        }
        let text = String::from_utf8(bytes).map_err(|_| {
            ReadToolError::MalformedUtf8(format!("Cannot decode UTF-8 file: {resource}"))
        })?;

        let lines: Vec<&str> = text.split('\n').collect();
        let line_count = lines.len();
        let start = offset.unwrap_or(1);
        if start == 0 || start > line_count {
            return Err(ReadToolError::OffsetOutOfRange(format!(
                "Offset {start} is out of range"
            )));
        }
        let start_index = start - 1;
        let end_index = match limit {
            Some(limit) => (start_index + limit).min(line_count),
            None => line_count,
        };
        let content = lines[start_index..end_index].join("\n");
        let truncated = end_index < line_count;
        let next = if truncated { Some(end_index + 1) } else { None };
        Ok(TextPage {
            content,
            truncated,
            next,
        })
    }

    /// List the entries of a directory.
    pub fn list(path: &str) -> Result<Vec<String>, ReadToolError> {
        let entries = std::fs::read_dir(path).map_err(|error| ReadToolError::FileSystem {
            method: "readDirectoryEntries".into(),
            message: error.to_string(),
        })?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| ReadToolError::FileSystem {
                method: "readDirectoryEntries".into(),
                message: error.to_string(),
            })?;
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        names.sort();
        Ok(names)
    }
}
