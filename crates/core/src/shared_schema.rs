//! Shared schema re-exports.
//!
//! Ports the observable behaviour of `packages/core/test/shared-schema.test.ts`:
//! core record schemas are the canonical shared schemas (construct and decode to
//! plain values), prompt equality is structural, a skill directory source keys as
//! `directory:<path>`, and workspace ids ascend with a `wrk_` prefix.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn make(text: impl Into<String>) -> CoreResult<Self> {
        Ok(Self { text: text.into() })
    }

    /// Construct a prompt from a user message.
    pub fn from_user_message(text: impl Into<String>) -> CoreResult<Self> {
        Self::make(text)
    }

    /// Decode a prompt from plain data.
    pub fn decode(value: &Value) -> CoreResult<Self> {
        let text = value
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("prompt missing text".into()))?;
        Ok(Self {
            text: text.to_string(),
        })
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
    pub fn decode(value: &Value) -> CoreResult<Self> {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("assistant text missing id".into()))?;
        let text = value
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("assistant text missing text".into()))?;
        Ok(Self {
            id: id.to_string(),
            text: text.to_string(),
        })
    }
}

/// Shared skill source helpers.
#[derive(Debug, Default)]
pub struct SkillSource;

impl SkillSource {
    /// The stable key for a skill source value.
    pub fn key(source: &Value) -> CoreResult<String> {
        let kind = source
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("skill source missing type".into()))?;
        let location = source
            .get("path")
            .or_else(|| source.get("url"))
            .or_else(|| source.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("skill source missing location".into()))?;
        Ok(format!("{kind}:{location}"))
    }
}

/// Workspace id helpers.
#[derive(Debug, Default)]
pub struct WorkspaceId;

static WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl WorkspaceId {
    /// An ascending workspace id derived from a seed.
    pub fn ascending(seed: &str) -> CoreResult<String> {
        let counter = WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(seed, &mut hasher);
        let seed_hash = std::hash::Hasher::finish(&hasher);
        Ok(format!("wrk_{millis:013x}{seed_hash:08x}{counter:08x}"))
    }
}
