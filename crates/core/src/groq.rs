//! Groq provider request lowering.
//!
//! Ports the observable behaviour of the Groq adapter in
//! `packages/core/test/provider-groq.test.ts`: a provider `reasoningEffort`
//! option is forwarded verbatim as `reasoning_effort`, including unknown values.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Groq request lowering.
#[derive(Debug, Default)]
pub struct GroqPlugin;

impl GroqPlugin {
    /// Apply Groq provider options onto a request body.
    pub fn apply_body(_options: &Value, _body: &mut Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented("groq::GroqPlugin::apply_body"))
    }
}
