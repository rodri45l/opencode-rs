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

/// A route the Rust server implements, tied to its reference `operationId`.
#[derive(Debug, Clone, Copy)]
pub struct ImplementedRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub operation_id: &'static str,
}

/// Routes implemented so far. Pinned to the contract by `tests/routes_contract.rs`.
pub const IMPLEMENTED_ROUTES: &[ImplementedRoute] = &[
    ImplementedRoute {
        method: "GET",
        path: path::HEALTH,
        operation_id: "v2.health.get",
    },
    ImplementedRoute {
        method: "GET",
        path: path::EVENT,
        operation_id: "v2.event.subscribe",
    },
    ImplementedRoute {
        method: "GET",
        path: path::EVENT_LEGACY,
        operation_id: "event.subscribe",
    },
];
