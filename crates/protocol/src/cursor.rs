//! Session list pagination cursors.
//!
//! Ports `packages/protocol/src/groups/session.ts` (`SessionsCursor`,
//! `SessionHistoryQuery`). A cursor is `base64url(json)` of the original query
//! plus an anchor; decoding a cursor restores the exact query to continue from.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use opencode_schema::{SessionId, WorkspaceId};
use serde::{Deserialize, Deserializer, Serialize};

/// Order for the first page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

/// Direction of the anchor relative to the current page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnchorDirection {
    Next,
    Previous,
}

/// Where the next page starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListAnchor {
    pub id: SessionId,
    pub time: i64,
    pub direction: AnchorDirection,
}

/// The decoded contents of a session list cursor.
///
/// The reference models this as a union of directory / project / all-query
/// shapes; the Rust type keeps the scope fields optional, which is sufficient
/// for round-tripping and is tightened when storage lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsCursorInput {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub workspace: Option<WorkspaceId>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub order: Option<Order>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subpath: Option<String>,
    pub anchor: ListAnchor,
}

/// Cursor error, mapping to `InvalidCursorError`.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Invalid cursor")]
pub struct InvalidCursor;

/// Encode a cursor input as `base64url(json)`.
pub fn encode_cursor(input: &SessionsCursorInput) -> Result<String, InvalidCursor> {
    let json = serde_json::to_vec(input).map_err(|_| InvalidCursor)?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

/// Decode a cursor produced by [`encode_cursor`].
pub fn decode_cursor(cursor: &str) -> Result<SessionsCursorInput, InvalidCursor> {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).map_err(|_| InvalidCursor)?;
    serde_json::from_slice(&bytes).map_err(|_| InvalidCursor)
}

/// Numeric query inputs decoded from strings, mirroring `NumberFromString`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionHistoryQuery {
    #[serde(
        default,
        deserialize_with = "opt_u32_from_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub after: Option<u32>,
    #[serde(
        default,
        deserialize_with = "opt_u32_from_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit: Option<u32>,
}

/// Maximum history page size, mirroring `SessionHistoryLimit`.
pub const HISTORY_LIMIT_MAX: u32 = 100;

impl SessionHistoryQuery {
    /// Validate `limit` is within the contract range.
    pub fn validate(&self) -> Result<(), InvalidCursor> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > HISTORY_LIMIT_MAX {
                return Err(InvalidCursor);
            }
        }
        Ok(())
    }
}

fn opt_u32_from_string<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u32),
    }

    let value = Option::<StringOrNumber>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        Some(StringOrNumber::String(s)) => {
            s.parse::<u32>().map(Some).map_err(serde::de::Error::custom)
        }
    }
}
