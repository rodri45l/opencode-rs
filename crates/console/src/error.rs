//! Typed errors for the console helpers.

/// A console helper failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConsoleError {
    #[error("invalid value: {0}")]
    InvalidValue(String),
}
