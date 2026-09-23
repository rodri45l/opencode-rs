//! Session info shapes and schema decoding.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/session.ts`
//! (the `Session.Info`/`GlobalInfo` wire shapes omit undefined optional keys)
//! and the session-domain schema decoders: valid input is accepted, invalid
//! input is rejected.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Token accounting for a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tokens {
    /// Input tokens.
    pub input: u64,
    /// Output tokens.
    pub output: u64,
    /// Reasoning tokens.
    pub reasoning: u64,
    /// Cache accounting.
    pub cache: TokenCache,
}

/// Prompt-cache token accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenCache {
    /// Cache reads.
    pub read: u64,
    /// Cache writes.
    pub write: u64,
}

/// A session summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Lines added.
    pub additions: i64,
    /// Lines removed.
    pub deletions: i64,
    /// Files touched.
    pub files: i64,
    /// Per-file diffs, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffs: Option<Value>,
}

/// A pending revert marker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Revert {
    /// The message to revert to.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Optional part to revert to.
    #[serde(rename = "partID", skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,
    /// Optional snapshot id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    /// Optional diff payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<Value>,
}

/// Session timestamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTime {
    /// Creation time.
    pub created: i64,
    /// Last update time, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<i64>,
    /// Compaction start time, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compacting: Option<i64>,
    /// Archive time, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<i64>,
}

/// A materialized session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session id.
    pub id: String,
    /// Human slug.
    pub slug: String,
    /// Owning project id.
    #[serde(rename = "projectID")]
    pub project_id: String,
    /// Workspace id, omitted when absent.
    #[serde(rename = "workspaceID", skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Instance directory.
    pub directory: String,
    /// Parent session id, omitted when absent.
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Summary, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<SessionSummary>,
    /// Accumulated cost.
    pub cost: f64,
    /// Token accounting.
    pub tokens: Tokens,
    /// Share info, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<Value>,
    /// Session title.
    pub title: String,
    /// Schema version.
    pub version: String,
    /// Free-form session metadata, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Timestamps.
    pub time: SessionTime,
    /// Session permission, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Value>,
    /// Pending revert, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert: Option<Revert>,
}

/// Project info attached to a global session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project id.
    pub id: String,
    /// Project name, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Worktree path.
    pub worktree: String,
}

/// A global session with its project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSessionInfo {
    /// The session fields.
    #[serde(flatten)]
    pub info: SessionInfo,
    /// The attached project, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectInfo>,
}

/// Encode a session to its wire JSON, omitting undefined optionals.
pub fn encode_info(info: &SessionInfo) -> Value {
    serde_json::to_value(info).expect("session info serializes")
}

/// Encode a global session to its wire JSON.
pub fn encode_global(info: &GlobalSessionInfo) -> Value {
    serde_json::to_value(info).expect("global session serializes")
}

/// Message timestamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTime {
    /// Creation time.
    pub created: i64,
    /// Completion time, omitted while in flight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<i64>,
}

/// A session message (minimum viable wire shape).
///
/// The reference models this as a union of user/assistant variants with many
/// optional fields; the Rust port keeps the identity fields typed and carries
/// the rest through untyped until their phases land.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Message id.
    pub id: String,
    /// Owning session id.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Message role (`user`/`assistant`).
    pub role: String,
    /// Timestamps, omitted when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<MessageTime>,
    /// Remaining message fields, carried verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// A message part (minimum viable wire shape).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Part {
    /// Part id.
    pub id: String,
    /// Owning session id.
    #[serde(rename = "sessionID")]
    pub session_id: String,
    /// Owning message id.
    #[serde(rename = "messageID")]
    pub message_id: String,
    /// Part discriminator.
    #[serde(rename = "type")]
    pub part_type: String,
    /// Remaining part fields, carried verbatim.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// A message together with its parts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageWithParts {
    /// Message metadata.
    pub info: Message,
    /// Ordered parts.
    pub parts: Vec<Part>,
}

/// Envelope returned by the v2 session list route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionsResponse {
    /// Page items.
    pub data: Vec<SessionInfo>,
    /// Pagination cursors.
    pub cursor: opencode_protocol::Cursor,
}

/// Envelope returned by the v2 session message list route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    /// Page items.
    pub data: Vec<Value>,
    /// Pagination cursors.
    pub cursor: opencode_protocol::Cursor,
}

/// Envelope returned by the v2 session history route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionHistory {
    /// Durable event page.
    pub data: Vec<Value>,
    /// Whether more events remain.
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

/// A schema decoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The decoder has not been ported yet.
    NotImplemented(&'static str),
    /// The input failed validation.
    Invalid(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Decode a named session-domain schema, returning the normalized value.
pub fn decode(schema: &str, value: &Value) -> Result<Value, DecodeError> {
    let _ = (schema, value);
    Err(DecodeError::NotImplemented("session::decode"))
}
