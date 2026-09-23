//! Web console (Phase 8).
//!
//! Pure console helper logic ported from `packages/console` (upstream 18ef3cc).

pub mod auth_redirect;
pub mod date;
pub mod error;
pub mod lite_usage;
pub mod muse_spark_policy;
pub mod pricing;
pub mod provider_usage;
pub mod rate_limiter;
pub mod request_body;
pub mod server_action;
pub mod subscription;
pub mod zen_model;

pub use error::ConsoleError;
