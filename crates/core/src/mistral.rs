//! Mistral provider request lowering.
//!
//! Ports the observable behaviour pinned by `packages/core/test/provider-mistral.test.ts`:
//! `promptCacheKey` lowers to `prompt_cache_key`, `reasoningEffort` lowers to
//! `reasoning_effort` (including unknown values), native thinking parts are
//! normalized, and a plain reasoning part collapses with adjacent text in
//! assistant history.

use serde_json::Value;

use crate::CoreResult;

/// Mistral request lowering.
#[derive(Debug, Default)]
pub struct MistralPlugin;

impl MistralPlugin {
    /// Apply Mistral provider options onto a request body.
    pub fn apply_request(options: &Value, body: &mut Value) -> CoreResult<()> {
        let Some(mistral) = options.get("mistral") else {
            return Ok(());
        };
        if let Some(cache_key) = mistral.get("promptCacheKey").and_then(Value::as_str) {
            if let Value::Object(map) = body {
                map.insert(
                    "prompt_cache_key".to_string(),
                    Value::String(cache_key.to_string()),
                );
            }
        }
        if let Some(effort) = mistral.get("reasoningEffort") {
            if let Value::Object(map) = body {
                map.insert("reasoning_effort".to_string(), effort.clone());
            }
        }
        Ok(())
    }

    /// Normalize a native thinking content part.
    pub fn normalize_thinking(content: &Value) -> CoreResult<Value> {
        Ok(serde_json::json!({
            "type": "reasoning",
            "text": "",
            "providerMetadata": { "mistral": { "thinking": content } },
        }))
    }

    /// Lower assistant history content to the wire shape.
    pub fn history_content(content: &Value) -> CoreResult<Value> {
        let Some(parts) = content.as_array() else {
            return Ok(content.clone());
        };
        // A plain reasoning part collapses with adjacent text into one string.
        if parts.iter().all(|part| {
            matches!(
                part.get("type").and_then(Value::as_str),
                Some("reasoning") | Some("text")
            )
        }) {
            let mut combined = String::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    combined.push_str(text);
                }
            }
            if !combined.is_empty() {
                return Ok(Value::String(combined));
            }
        }
        Ok(content.clone())
    }
}
