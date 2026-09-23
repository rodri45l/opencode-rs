//! `which` command lookup.
//!
//! Ports the observable behaviour of `packages/core/src/util/which.ts`: locate an
//! executable on a PATH (honoring PATHEXT on Windows), returning the first match
//! or `None`.

use std::path::PathBuf;

use crate::{CoreError, CoreResult};

/// Command lookup.
#[derive(Debug, Default)]
pub struct Which;

impl Which {
    /// Locate `command` on `path`, optionally using `pathext`.
    pub fn which(
        _command: &str,
        _path: &str,
        _pathext: Option<&str>,
    ) -> CoreResult<Option<PathBuf>> {
        Err(CoreError::NotImplemented("which::Which::which"))
    }
}
