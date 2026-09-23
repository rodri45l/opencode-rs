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
        let directory = std::env::temp_dir().join("opencode");
        std::fs::create_dir_all(&directory)
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        Ok(directory)
    }

    /// Build the global path bundle.
    pub fn make() -> CoreResult<GlobalPath> {
        Ok(GlobalPath { tmp: Self::tmp()? })
    }
}
