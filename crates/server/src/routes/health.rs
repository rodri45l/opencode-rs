//! `GET /api/health`.

use crate::state::AppState;
use axum::routing::get;
use axum::{Json, Router};
use opencode_protocol::routes::path;
use serde::Serialize;

/// Health response body. The contract fixes `healthy` to `true`.
#[derive(Debug, Serialize)]
pub struct Health {
    pub healthy: bool,
}

/// Report that the server is ready.
pub async fn get_health() -> Json<Health> {
    Json(Health { healthy: true })
}

/// Routes for the health group.
pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route(path::HEALTH, get(get_health))
}
