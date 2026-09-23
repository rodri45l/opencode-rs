//! Global on-disk locations.
//!
//! Ports `packages/core/src/global.ts`: a single global directory rooted under
//! the system temp/state directory, created when the module is loaded.

use std::path::PathBuf;

use crate::{CoreError, CoreResult};

/// Well-known global paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPath {
    /// Scratch directory (`<tmp>/opencode`).
    pub tmp: PathBuf,
}

/// Global path accessors.
#[derive(Debug, Default)]
pub struct Global;

impl Global {
    /// The global scratch directory, `<system-temp>/opencode`.
    pub fn tmp() -> CoreResult<PathBuf> {
        Err(CoreError::NotImplemented("global::Global::tmp"))
    }

    /// Build the global path bundle.
    pub fn make() -> CoreResult<GlobalPath> {
        Err(CoreError::NotImplemented("global::Global::make"))
    }
}
