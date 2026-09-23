//! Generated Rust client SDK.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.

pub mod embedded;
pub mod import_boundaries;

/// Path parameters for `GET /api/session/{sessionID}/history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2SessionHistoryPath {
    pub session_id: String,
}

/// Query parameters for `GET /api/session/{sessionID}/history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2SessionHistoryQuery {
    pub after: i64,
    pub limit: i64,
}

/// Request data for `GET /api/session/{sessionID}/history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V2SessionHistoryData {
    pub path: V2SessionHistoryPath,
    pub query: V2SessionHistoryQuery,
    pub url: String,
}
