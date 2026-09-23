//! Shared schema re-exports.
//!
//! Ports the observable behaviour of `packages/core/test/shared-schema.test.ts`:
//! core record schemas are the canonical shared schemas (construct and decode to
//! plain values), prompt equality is structural, a skill directory source keys as
//! `directory:<path>`, and workspace ids ascend with a `wrk_` prefix.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A user prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// Prompt text.
    pub text: String,
}

impl Prompt {
    /// Construct a prompt.
    pub fn make(_text: impl Into<String>) -> CoreResult<Self> {
        Err(CoreError::NotImplemented("shared_schema::Prompt::make"))
    }

    /// Construct a prompt from a user message.
    pub fn from_user_message(_text: impl Into<String>) -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "shared_schema::Prompt::from_user_message",
        ))
    }

    /// Decode a prompt from plain data.
    pub fn decode(_value: &Value) -> CoreResult<Self> {
        Err(CoreError::NotImplemented("shared_schema::Prompt::decode"))
    }
}

/// An assistant text part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantText {
    /// Part id.
    pub id: String,
    /// Text content.
    pub text: String,
}

impl AssistantText {
    /// Decode an assistant text part from plain data.
    pub fn decode(_value: &Value) -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "shared_schema::AssistantText::decode",
        ))
    }
}

/// Shared skill source helpers.
#[derive(Debug, Default)]
pub struct SkillSource;

impl SkillSource {
    /// The stable key for a skill source value.
    pub fn key(_source: &Value) -> CoreResult<String> {
        Err(CoreError::NotImplemented("shared_schema::SkillSource::key"))
    }
}

/// Workspace id helpers.
#[derive(Debug, Default)]
pub struct WorkspaceId;

impl WorkspaceId {
    /// An ascending workspace id derived from a seed.
    pub fn ascending(_seed: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "shared_schema::WorkspaceId::ascending",
        ))
    }
}
