//! Models.dev catalog cache.
//!
//! Ports the on-disk cache behaviour of `packages/core/src/models-dev.ts`:
//! `get` serves the cached provider catalog, an absent cache yields an empty
//! catalog when fetching is disabled, and the in-memory result is cached across
//! calls until an explicit refresh.

use std::path::PathBuf;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Models.dev catalog cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelsDev {
    cache_path: PathBuf,
}

impl ModelsDev {
    /// Build a cache bound to `cache_path`.
    pub fn with_cache_path(cache_path: impl Into<PathBuf>) -> Self {
        Self {
            cache_path: cache_path.into(),
        }
    }

    /// The backing cache file.
    pub fn cache_path(&self) -> &PathBuf {
        &self.cache_path
    }

    /// The provider catalog.
    pub fn get(&self) -> CoreResult<Value> {
        Err(CoreError::NotImplemented("models_dev::ModelsDev::get"))
    }

    /// Refresh the catalog, optionally forcing a fetch.
    pub fn refresh(&self, _force: bool) -> CoreResult<()> {
        Err(CoreError::NotImplemented("models_dev::ModelsDev::refresh"))
    }
}
