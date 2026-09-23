//! Snowflake Cortex provider plugin.
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/snowflake-cortex.ts`: tokens resolve from
//! `SNOWFLAKE_CORTEX_PAT`/`SNOWFLAKE_CORTEX_TOKEN` with an options fallback,
//! `max_tokens` is rewritten to `max_completion_tokens`, a `400` "Conversation
//! complete" body settles as a stop response, streaming chunks rewrite an empty
//! role to `assistant`, and the plugin is registered before the
//! openai-compatible fallback.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// Snowflake Cortex SDK options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CortexOptions {
    /// PAT fallback.
    pub pat: Option<String>,
    /// OAuth token fallback.
    pub token: Option<String>,
    /// API key fallback.
    pub api_key: Option<String>,
}

/// The Snowflake Cortex provider plugin.
#[derive(Debug, Default)]
pub struct SnowflakeCortexPlugin;

impl SnowflakeCortexPlugin {
    /// The registry index of a built-in provider plugin id.
    pub fn registry_index(id: &str) -> CoreResult<usize> {
        let order = [
            "google-vertex",
            "snowflake-cortex",
            "kilo",
            "zenmux",
            "openai-compatible",
        ];
        Ok(order
            .iter()
            .position(|entry| *entry == id)
            .unwrap_or(usize::MAX))
    }

    /// Resolve the bearer token from the environment, then options.
    pub fn resolve_token(
        env: &BTreeMap<String, String>,
        options: &CortexOptions,
    ) -> CoreResult<Option<String>> {
        Ok(env
            .get("SNOWFLAKE_CORTEX_PAT")
            .filter(|value| !value.is_empty())
            .cloned()
            .or_else(|| {
                env.get("SNOWFLAKE_CORTEX_TOKEN")
                    .filter(|value| !value.is_empty())
                    .cloned()
            })
            .or_else(|| options.pat.clone().filter(|value| !value.is_empty()))
            .or_else(|| options.token.clone().filter(|value| !value.is_empty()))
            .or_else(|| options.api_key.clone().filter(|value| !value.is_empty())))
    }

    /// Whether usage is requested on the SDK options.
    pub fn include_usage(
        env: &BTreeMap<String, String>,
        options: &CortexOptions,
    ) -> CoreResult<bool> {
        Ok(Self::resolve_token(env, options)?.is_some())
    }

    /// Rewrite `max_tokens` to `max_completion_tokens`, preserving other bodies.
    pub fn rewrite_request_body(body: &str) -> CoreResult<String> {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body) else {
            return Ok(body.to_string());
        };
        let Some(map) = value.as_object_mut() else {
            return Ok(body.to_string());
        };
        if let Some(max_tokens) = map.remove("max_tokens") {
            map.insert("max_completion_tokens".to_string(), max_tokens);
        }
        serde_json::to_string(&value).map_err(|error| CoreError::Message(error.to_string()))
    }

    /// Normalize a Cortex response: a `400` "Conversation complete" becomes a
    /// `200` stop response.
    pub fn rewrite_response(status: u16, body: &str) -> CoreResult<(u16, String)> {
        if status != 400 {
            return Ok((status, body.to_string()));
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
            return Ok((status, body.to_string()));
        };
        let message = value.get("message").and_then(serde_json::Value::as_str);
        if message != Some("Conversation complete") {
            return Ok((status, body.to_string()));
        }
        let rewritten = serde_json::json!({
            "choices": [{ "finish_reason": "stop", "message": { "role": "assistant", "content": "" } }],
        });
        Ok((
            200,
            serde_json::to_string(&rewritten)
                .map_err(|error| CoreError::Message(error.to_string()))?,
        ))
    }

    /// Rewrite an empty streaming `role` to `assistant`.
    pub fn rewrite_streaming_role(chunk: &str) -> CoreResult<String> {
        Ok(chunk.replace("\"role\":\"\"", "\"role\":\"assistant\""))
    }
}
