//! xAI provider request lowering.
//!
//! Ports the observable behaviour pinned by
//! `packages/core/test/provider-xai-responses.test.ts`: the responses API lowers
//! `promptCacheKey` to `prompt_cache_key` and a `reasoningEffort` to a
//! `reasoning.effort` object, while the chat API lowers `reasoningEffort` to a
//! flat `reasoning_effort`.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Which xAI API a request targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XaiApi {
    /// The responses API.
    Responses,
    /// The chat completions API.
    Chat,
}

/// xAI request lowering.
#[derive(Debug, Default)]
pub struct XaiResponsesPlugin;

impl XaiResponsesPlugin {
    /// Apply xAI provider options onto a request body for the given API.
    pub fn apply_options(_api: XaiApi, _options: &Value, _body: &mut Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "xai_responses::XaiResponsesPlugin::apply_options",
        ))
    }
}
