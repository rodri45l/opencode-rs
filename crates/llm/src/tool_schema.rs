//! Protocol-specific tool schema projections.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};

/// Provider-specific JSON Schema projections.
pub struct ToolSchemaProjection;

impl ToolSchemaProjection {
    /// Moonshot: strip `$ref` siblings and flatten tuple arrays.
    pub fn moonshot(_schema: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("moonshot tool schema projection"))
    }

    /// Gemini: normalise numeric enums, dangling required, untyped arrays.
    pub fn gemini(_schema: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("gemini tool schema projection"))
    }

    /// OpenAI: flatten top-level object unions into one object schema.
    pub fn open_ai(_schema: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("openai tool schema projection"))
    }
}
