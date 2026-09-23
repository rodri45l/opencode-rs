//! Slack, enterprise, and container integrations.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.

pub mod share;
pub mod storage;

pub use share::{Share, ShareData, ShareError, ShareInfo, ShareRef};
pub use storage::{ListOptions, Storage};
