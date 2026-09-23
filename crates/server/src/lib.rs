//! The opencode HTTP server.
//!
//! Phase 1 exposes the health endpoint and the SSE event stream. The router is
//! assembled here and is intentionally the only place that knows the full path
//! surface.

pub mod agent;
pub mod auth;
pub mod error;
pub mod lsp_config;
pub mod mcp_oauth;
pub mod mdns;
pub mod options;
pub mod permission;
pub mod port;
pub mod proxy_util;
pub mod retry;
pub mod routes;
pub mod session;
pub mod state;
pub mod structured_output;
pub mod system_prompt;
pub mod tools;
pub mod truncate;
pub mod websearch;
pub mod workspace_routing;

pub use auth::AuthConfig;
pub use error::ApiErrorResponse;
pub use options::{router_with_options, ServerOptions};
pub use state::AppState;

use axum::Router;
use tower_http::trace::TraceLayer;

/// Build the application router.
pub fn router(state: AppState) -> Router {
    Router::<AppState>::new()
        .merge(routes::health::router())
        .merge(routes::event::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
