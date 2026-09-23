//! The opencode HTTP server.
//!
//! Phase 1 exposes the health endpoint and the SSE event stream. The router is
//! assembled here and is intentionally the only place that knows the full path
//! surface.

pub mod acp_config_option;
pub mod acp_content;
pub mod agent;
pub mod auth;
pub mod config_entry_name;
pub mod config_markdown;
pub mod data_url;
pub mod error;
pub mod fs_util;
pub mod html_util;
pub mod ide;
pub mod lsp_config;
pub mod lsp_index;
pub mod lsp_jdtls;
pub mod mcp_oauth;
pub mod mdns;
pub mod message_page;
pub mod named_error;
pub mod options;
pub mod patch;
pub mod permission;
pub mod port;
pub mod provider_util;
pub mod proxy_util;
pub mod repository;
pub mod retry;
pub mod routes;
pub mod session;
pub mod session_store;
pub mod session_usage;
pub mod state;
pub mod structured_output;
pub mod system_prompt;
pub mod timeout_util;
pub mod tools;
pub mod truncate;
pub mod websearch;
pub mod wildcard;
pub mod workspace_routing;

pub use auth::AuthConfig;
pub use error::ApiErrorResponse;
pub use options::{router_with_options, ServerOptions};
pub use state::AppState;

use axum::Router;

/// Build the application router.
pub fn router(state: AppState) -> Router {
    options::router_with_options(state, ServerOptions::new())
}
