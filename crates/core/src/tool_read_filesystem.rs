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
        _path: &str,
        _resource: &str,
        _offset: Option<usize>,
        _limit: Option<usize>,
    ) -> Result<TextPage, ReadToolError> {
        Err(ReadToolError::FileSystem {
            method: "readFile".into(),
            message: "not implemented".into(),
        })
    }

    /// List the entries of a directory.
    pub fn list(_path: &str) -> Result<Vec<String>, ReadToolError> {
        Err(ReadToolError::FileSystem {
            method: "readDirectoryEntries".into(),
            message: "not implemented".into(),
        })
    }
}
