//! Groq provider request lowering.
//!
//! Ports the observable behaviour of the Groq adapter in
//! `packages/core/test/provider-groq.test.ts`: a provider `reasoningEffort`
//! option is forwarded verbatim as `reasoning_effort`, including unknown values.

use serde_json::Value;

use crate::CoreResult;

/// Groq request lowering.
#[derive(Debug, Default)]
pub struct GroqPlugin;

impl GroqPlugin {
    /// Apply Groq provider options onto a request body.
    pub fn apply_body(options: &Value, body: &mut Value) -> CoreResult<()> {
        if let Some(effort) = options
            .get("groq")
            .and_then(|groq| groq.get("reasoningEffort"))
        {
            if let Value::Object(map) = body {
                map.insert("reasoning_effort".to_string(), effort.clone());
            }
        }
        Ok(())
    }
}
