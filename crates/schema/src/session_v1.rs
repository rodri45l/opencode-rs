//! Compatibility entrypoint for the isolated V1 session subtree.
//!
//! Mirrors `packages/schema/src/session-v1.ts`, which re-exports
//! `./v1/session`.

pub use crate::v1::session::*;
