//! Default ignore matching.
//!
//! Ports the observable behaviour of `packages/core/src/filesystem/ignore.ts`:
//! the default ignore set matches `node_modules` at any depth, with or without
//! a trailing slash and with nested descendants.

use crate::{CoreError, CoreResult};

/// Default ignore matching.
#[derive(Debug, Default)]
pub struct Ignore;

impl Ignore {
    /// Whether `path` matches the default ignore set.
    pub fn match_path(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("fs_ignore::Ignore::match_path"))
    }
}
