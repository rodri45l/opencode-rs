//! Usage and cost statistics.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.

pub mod home;
pub mod inference;
pub mod model_normalization;
pub mod r2_sql;

/// Errors surfaced by statistics helpers whose backing store is not yet ported.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StatsError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}
