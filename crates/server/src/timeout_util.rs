//! Promise-style timeout helper.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/timeout.ts`:
//! resolve with the value when the future completes in time, otherwise reject
//! with `Operation timed out after <ms>ms`.

use std::future::Future;

/// The error returned when a future exceeds its deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeoutError {
    /// The future exceeded the deadline.
    TimedOut(u64),
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimedOut(ms) => write!(f, "Operation timed out after {ms}ms"),
        }
    }
}

impl std::error::Error for TimeoutError {}

/// Await `future`, failing if it does not complete within `millis`.
pub async fn with_timeout<F, T>(future: F, millis: u64) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    match tokio::time::timeout(std::time::Duration::from_millis(millis), future).await {
        Ok(value) => Ok(value),
        Err(_) => Err(TimeoutError::TimedOut(millis)),
    }
}
