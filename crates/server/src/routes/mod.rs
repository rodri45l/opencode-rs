//! Route modules.

pub mod event;
pub mod health;
pub mod misc;
pub mod openapi;
pub mod pty_v2;
pub mod session;
pub mod session_v2;

use crate::state::AppState;
use axum::Router;

/// The full route surface, without auth/CORS/compression layers or state.
pub fn base_router() -> Router<AppState> {
    Router::<AppState>::new()
        .merge(health::router())
        .merge(event::router())
        .merge(session::router())
        .merge(session_v2::router())
        .merge(pty_v2::router())
        .merge(misc::router())
}
