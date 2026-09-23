//! Error surface for the LLM crate.
//!
//! Every behavioural entry point returns a typed [`LlmError`]; until an adapter
//! lands the body returns [`LlmError::NotImplemented`] so the red-first tests
//! fail with a clear value instead of a panic.

use std::fmt;

/// Result alias used across the crate.
pub type LlmResult<T> = Result<T, LlmError>;

/// A provider HTTP failure with redacted diagnostics.
#[derive(Debug, Clone)]
pub struct ProviderHttpError {
    /// Human-readable summary, e.g. `HTTP 400: ...`.
    pub message: String,
    /// Optional context-overflow classification.
    pub classification: Option<&'static str>,
    /// Whether the executor may retry.
    pub retryable: bool,
    /// Redacted, truncated diagnostics included in `Debug` output.
    pub diagnostics: String,
}

/// A classified provider/route failure.
#[derive(Debug)]
pub enum LlmError {
    /// The module has not been implemented yet.
    NotImplemented(&'static str),
    /// A generic provider error.
    Provider(String),
    /// The request was rejected before it reached the provider.
    InvalidRequest(String),
    /// A provider HTTP response failure.
    ProviderHttp(Box<ProviderHttpError>),
}

impl LlmError {
    /// Context-overflow classification carried by provider errors, if any.
    pub fn classification(&self) -> Option<&'static str> {
        match self {
            LlmError::ProviderHttp(error) => error.classification,
            _ => None,
        }
    }

    /// Whether the executor may retry the request.
    pub fn retryable(&self) -> bool {
        match self {
            LlmError::ProviderHttp(error) => error.retryable,
            _ => false,
        }
    }

    /// Build a context-overflow invalid-request error.
    pub fn invalid(message: impl Into<String>) -> Self {
        LlmError::InvalidRequest(message.into())
    }
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NotImplemented(what) => write!(f, "not implemented: {what}"),
            LlmError::Provider(message) => write!(f, "provider error: {message}"),
            LlmError::InvalidRequest(message) => write!(f, "invalid request: {message}"),
            LlmError::ProviderHttp(error) => write!(f, "{}", error.message),
        }
    }
}

impl std::error::Error for LlmError {}
