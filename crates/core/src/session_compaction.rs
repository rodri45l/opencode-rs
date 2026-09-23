//! Session compaction prompt and tool-content serialization.
//!
//! Ports the observable behaviour of `packages/core/src/session/compaction.ts`:
//! the compaction prompt embeds the conversation (and any prior summary) and
//! asks for an anchored summary with a fixed work-state shape; tool media is
//! described without embedding base64.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Session compaction helpers.
#[derive(Debug, Default)]
pub struct SessionCompaction;

impl SessionCompaction {
    /// Build the compaction prompt.
    pub fn build_prompt(
        _context: &[String],
        _previous_summary: Option<&str>,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "session_compaction::SessionCompaction::build_prompt",
        ))
    }

    /// Serialize tool content parts for compaction, describing media.
    pub fn serialize_tool_content(_parts: &[Value]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "session_compaction::SessionCompaction::serialize_tool_content",
        ))
    }
}
