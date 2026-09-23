//! Compatibility entrypoint for the isolated V1 legacy-event subtree.
//!
//! Mirrors `packages/schema/src/legacy-event.ts`, which re-exports
//! `./v1/legacy-event`.

pub use crate::v1::legacy_event::*;
