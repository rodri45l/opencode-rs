//! Compatibility entrypoint for the isolated V1 permission subtree.
//!
//! Mirrors `packages/schema/src/permission-v1.ts`, which re-exports
//! `./v1/permission`.

pub use crate::v1::permission::*;
