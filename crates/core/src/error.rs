//! Typed errors returned by core services.
//!
//! Modules that have not landed yet return [`CoreError::NotImplemented`] from
//! their stubs, so tests ported ahead of implementation fail with a clear value
//! rather than panicking (see `docs/PORTING.md`).

use std::fmt;

/// Error type shared by the core crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
    /// A conditional create found the target already present.
    TargetExists(String),
    /// A conditional write found the target content stale.
    StaleContent(String),
    /// A filesystem operation failed.
    FileSystem(String),
    /// Input failed validation.
    Invalid(String),
    /// A generic diagnostic message.
    Message(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::TargetExists(path) => write!(f, "target exists: {path}"),
            Self::StaleContent(path) => write!(f, "stale content: {path}"),
            Self::FileSystem(message) => write!(f, "filesystem error: {message}"),
            Self::Invalid(message) => f.write_str(message),
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for CoreError {}

/// Convenience alias for fallible core operations.
pub type CoreResult<T> = Result<T, CoreError>;
