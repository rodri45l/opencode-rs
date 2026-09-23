//! Canonical route paths.
//!
//! Paths are copied verbatim from the reference `openapi.json`. Only the routes
//! implemented per phase live here; the full 162-path surface is tracked in
//! `docs/PLAN.md`.

/// Static route paths.
pub mod path {
    /// Health check.
    pub const HEALTH: &str = "/api/health";
    /// Server event stream (SSE).
    pub const EVENT: &str = "/api/event";
    /// Legacy event stream (SSE).
    pub const EVENT_LEGACY: &str = "/event";

    /// List sessions.
    pub const SESSION: &str = "/api/session";
}

/// Path for a specific session.
pub fn session(id: &str) -> String {
    format!("{}/{id}", path::SESSION)
}

/// Cursor-based pagination cursor as returned on responses.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub previous: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub next: Option<String>,
}

/// Standard list envelope used by paginated endpoints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cursor: Option<Cursor>,
}
