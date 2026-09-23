//! Process preload environment.
//!
//! Ports the observable behaviour of `packages/core/test/preload.test.ts` (and
//! `packages/core/src/preload.ts`): loading the module disables public npm
//! security audits by exporting `NPM_CONFIG_AUDIT=false`.

use crate::{CoreError, CoreResult};

/// Environment applied when the core runtime is preloaded.
#[derive(Debug, Default)]
pub struct Preload;

impl Preload {
    /// The environment variables the preload installs.
    pub fn environment() -> CoreResult<Vec<(String, String)>> {
        Err(CoreError::NotImplemented("preload::Preload::environment"))
    }
}
