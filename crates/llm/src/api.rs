//! Top-level `LLM` namespace.

use serde_json::Value;

use crate::error::LlmResult;
use crate::provider_error::is_context_overflow;
use crate::schema::{build_request, merge_provider_options};

/// App-facing LLM runtime helpers.
pub struct LLM;

impl LLM {
    /// Build a canonical request from ergonomic input.
    pub fn request(input: Value) -> Value {
        build_request(input).unwrap_or(Value::Null)
    }

    /// Apply a patch to a canonical request.
    pub fn update_request(base: Value, patch: Value) -> LlmResult<Value> {
        crate::schema::LLMRequest::update(base, patch)
    }

    /// Deep-merge provider option layers.
    pub fn merge_provider_options(layers: Vec<Value>) -> LlmResult<Value> {
        let items: Vec<Option<Value>> = layers.into_iter().map(Some).collect();
        Ok(merge_provider_options(&items).unwrap_or(Value::Null))
    }

    /// Force a synthetic tool call and decode the structured result.
    pub fn generate_object(input: Value) -> LlmResult<Value> {
        crate::generate_object::generate_object(input)
    }

    /// Classify a provider message as a context-overflow error.
    pub fn is_context_overflow(message: &str) -> LlmResult<bool> {
        Ok(is_context_overflow(message))
    }
}
