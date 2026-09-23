//! Library surface for the `opencode-rs` CLI.
//!
//! The binary is a thin wrapper; command handlers live here so they can be
//! unit-tested independently of process I/O.

pub mod handlers;
