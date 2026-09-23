//! Top-level `LLM` namespace.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// App-facing LLM runtime helpers.
pub struct LLM;

impl LLM {
    /// Build a canonical request from ergonomic input.
    pub fn request(input: Value) -> Value {
        input
    }

    /// Apply a patch to a canonical request.
    pub fn update_request(_base: Value, _patch: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("llm update request"))
    }

    /// Deep-merge provider option layers.
    pub fn merge_provider_options(_layers: Vec<Value>) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("merge provider options"))
    }

    /// Force a synthetic tool call and decode the structured result.
    pub fn generate_object(_input: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("generate object"))
    }

    /// Classify a provider message as a context-overflow error.
    pub fn is_context_overflow(_message: &str) -> LlmResult<bool> {
        Err(LlmError::NotImplemented("context overflow classification"))
    }
}
