//! The opencode HTTP server.
//!
//! Phase 1 exposes the health endpoint and the SSE event stream. The router is
//! assembled here and is intentionally the only place that knows the full path
//! surface.

pub mod auth;
pub mod error;
pub mod options;
pub mod routes;
pub mod state;

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
