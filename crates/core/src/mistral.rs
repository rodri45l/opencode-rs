//! Mistral provider request lowering.
//!
//! Ports the observable behaviour pinned by `packages/core/test/provider-mistral.test.ts`:
//! `promptCacheKey` lowers to `prompt_cache_key`, `reasoningEffort` lowers to
//! `reasoning_effort` (including unknown values), native thinking parts are
//! normalized, and a plain reasoning part collapses with adjacent text in
//! assistant history.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Mistral request lowering.
#[derive(Debug, Default)]
pub struct MistralPlugin;

impl MistralPlugin {
    /// Apply Mistral provider options onto a request body.
    pub fn apply_request(_options: &Value, _body: &mut Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "mistral::MistralPlugin::apply_request",
        ))
    }

    /// Normalize a native thinking content part.
    pub fn normalize_thinking(_content: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "mistral::MistralPlugin::normalize_thinking",
        ))
    }

    /// Lower assistant history content to the wire shape.
    pub fn history_content(_content: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "mistral::MistralPlugin::history_content",
        ))
    }
}
