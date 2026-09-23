//! GitHub Copilot Responses conversions (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/github-copilot/responses/convert-to-openai-responses-input.ts`
//! and the `copilot` provider-metadata namespace of
//! `.../openai-responses-language-model.ts`: a tool call echoes a stale
//! `copilot.itemId` as the `function_call` id (omitted once stripped), a
//! reasoning part is preserved only when it carries a `copilot.itemId` (and
//! otherwise dropped with a warning), a user file part reads `imageDetail` from
//! the `copilot` namespace, and generated item/response metadata is attached
//! under the `copilot` namespace rather than `openai`. The `LanguageModelV3`
//! network/runtime and the mock fetch are dropped.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// The provider-metadata namespace used by the Copilot Responses model.
pub const METADATA_NAMESPACE: &str = "copilot";

/// A converted Responses input plus any warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedInput {
    /// The OpenAI Responses input items.
    pub input: Vec<Value>,
    /// Conversion warnings.
    pub warnings: Vec<Value>,
}

/// The warning emitted when a reasoning part has no Copilot item id.
pub const REASONING_WARNING: &str = "Non-OpenAI reasoning parts are not supported";

/// Convert a `LanguageModelV3` prompt array into Responses input items.
pub fn convert_to_openai_responses_input(_prompt: &Value) -> CoreResult<ConvertedInput> {
    Err(CoreError::NotImplemented(
        "github_copilot_responses::convert_to_openai_responses_input",
    ))
}

/// Provider metadata for a generated item, namespaced under `copilot`.
#[derive(Debug, Default)]
pub struct ResponsesMetadata;

impl ResponsesMetadata {
    /// Provider metadata for a reasoning item.
    pub fn reasoning(_item_id: &str, _encrypted_content: Option<&str>) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "github_copilot_responses::ResponsesMetadata::reasoning",
        ))
    }

    /// Provider metadata for a message or function-call item.
    pub fn item(_item_id: &str) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "github_copilot_responses::ResponsesMetadata::item",
        ))
    }

    /// Response-level provider metadata.
    pub fn response(_response_id: &str) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "github_copilot_responses::ResponsesMetadata::response",
        ))
    }
}

/// The metadata for a reasoning item carrying an encrypted summary.
pub fn reasoning_metadata(item_id: &str, encrypted_content: Option<&str>) -> Value {
    let mut fields = serde_json::Map::new();
    fields.insert("itemId".into(), Value::String(item_id.to_string()));
    if let Some(encrypted) = encrypted_content {
        fields.insert(
            "reasoningEncryptedContent".into(),
            Value::String(encrypted.to_string()),
        );
    }
    let mut namespaced = serde_json::Map::new();
    namespaced.insert(METADATA_NAMESPACE.to_string(), Value::Object(fields));
    Value::Object(namespaced)
}
