//! xAI provider request lowering.
//!
//! Ports the observable behaviour pinned by
//! `packages/core/test/provider-xai-responses.test.ts`: the responses API lowers
//! `promptCacheKey` to `prompt_cache_key` and a `reasoningEffort` to a
//! `reasoning.effort` object, while the chat API lowers `reasoningEffort` to a
//! flat `reasoning_effort`.

use serde_json::{Map, Value};

use crate::CoreResult;

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
    pub fn apply_options(api: XaiApi, options: &Value, body: &mut Value) -> CoreResult<()> {
        let Some(xai) = options.get("xai") else {
            return Ok(());
        };
        if let Some(cache_key) = xai.get("promptCacheKey").and_then(Value::as_str) {
            set(
                body,
                "prompt_cache_key",
                Value::String(cache_key.to_string()),
            );
        }
        if let Some(effort) = xai.get("reasoningEffort") {
            match api {
                XaiApi::Responses => {
                    let mut reasoning = Map::new();
                    reasoning.insert("effort".to_string(), effort.clone());
                    set(body, "reasoning", Value::Object(reasoning));
                }
                XaiApi::Chat => {
                    set(body, "reasoning_effort", effort.clone());
                }
            }
        }
        Ok(())
    }
}

fn set(body: &mut Value, key: &str, value: Value) {
    if let Value::Object(map) = body {
        map.insert(key.to_string(), value);
    }
}
