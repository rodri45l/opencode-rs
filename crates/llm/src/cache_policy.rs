//! Automatic cache-breakpoint placement.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// Apply the request's cache policy, returning the lowered request.
pub fn apply_cache_policy(_request: Value) -> LlmResult<Value> {
    Err(LlmError::NotImplemented("apply cache policy"))
}
