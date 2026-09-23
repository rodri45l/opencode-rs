//! Server construction options.
//!
//! `router` builds the default (unauthenticated, no-CORS) application. Tests
//! that pin auth and CORS behaviour build the router through
//! [`router_with_options`] so the configuration is explicit at the call site.

use crate::auth::AuthConfig;
use crate::state::AppState;
use axum::Router;

/// Options that shape a server instance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerOptions {
    /// Basic-auth configuration.
    pub auth: AuthConfig,
    /// Extra origins permitted by the CORS layer.
    pub cors: Vec<String>,
}

impl ServerOptions {
    /// Default options: auth disabled, no custom CORS origins.
    pub fn new() -> Self {
        Self::default()
    }

    /// Options with basic auth enabled.
    pub fn with_auth(auth: AuthConfig) -> Self {
        Self {
            auth,
            ..Self::default()
        }
    }

    /// Options with custom CORS origins.
    pub fn with_cors(origins: Vec<String>) -> Self {
        Self {
            cors: origins,
            ..Self::default()
        }
    }
}

/// Build the application router honouring `options`.
///
/// The auth/CORS middleware is not ported yet, so this currently delegates to
/// the default [`crate::router`]. Tests that exercise those layers stay red
/// until the middleware lands.
pub fn router_with_options(state: AppState, _options: ServerOptions) -> Router {
    crate::router(state)
}
