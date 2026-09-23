//! Legacy event schema compatibility (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/v1/session.ts`'s
//! `APIError`: constructing it and serializing with `toObject` yields
//! `{ name: "APIError", data: { message, isRetryable } }`. The cross-package
//! `SessionV1` definition identity assertions are dropped (they are about module
//! identity, not behaviour).

use serde_json::{json, Value};

use crate::{CoreError, CoreResult};

/// A legacy session API error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The error message.
    pub message: String,
    /// Whether the operation may be retried.
    pub is_retryable: bool,
}

impl ApiError {
    /// Construct an error.
    pub fn new(_message: &str, _is_retryable: bool) -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "legacy_event_schema::ApiError::new",
        ))
    }

    /// Serialize to the canonical object shape.
    pub fn to_object(&self) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "legacy_event_schema::ApiError::to_object",
        ))
    }
}

/// The canonical object name for [`ApiError`].
pub const API_ERROR_NAME: &str = "APIError";

/// Build the object shape directly from fields.
pub fn api_error_object(message: &str, is_retryable: bool) -> Value {
    json!({
        "name": API_ERROR_NAME,
        "data": { "message": message, "isRetryable": is_retryable },
    })
}
