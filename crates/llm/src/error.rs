//! Error surface for the LLM crate.
//!
//! Every behavioural entry point returns a typed [`LlmError`]; until an adapter
//! lands the body returns [`LlmError::NotImplemented`] so the red-first tests
//! fail with a clear value instead of a panic.

/// Result alias used across the crate.
pub type LlmResult<T> = Result<T, LlmError>;

/// A classified provider/route failure.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// The module has not been implemented yet.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    /// A generic provider error.
    #[error("provider error: {0}")]
    Provider(String),
    /// The request was rejected before it reached the provider.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl LlmError {
    /// Context-overflow classification carried by provider errors, if any.
    pub fn classification(&self) -> Option<&'static str> {
        match self {
            LlmError::InvalidRequest(_) | LlmError::NotImplemented(_) | LlmError::Provider(_) => {
                None
            }
        }
    }

    /// Whether the executor may retry the request.
    pub fn retryable(&self) -> bool {
        false
    }
}
