//! GitHub Copilot message conversion.
//!
//! Ports the observable behaviour of
//! `packages/core/src/github-copilot/chat/convert-to-openai-compatible-chat-messages.ts`:
//! convert canonical prompt messages into OpenAI-compatible chat messages,
//! including image URL parts, tool calls/results, and copilot reasoning fields.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Convert canonical messages to OpenAI-compatible chat messages.
pub fn convert_to_openai_compatible_chat_messages(_messages: &[Value]) -> CoreResult<Vec<Value>> {
    Err(CoreError::NotImplemented(
        "copilot_convert::convert_to_openai_compatible_chat_messages",
    ))
}
