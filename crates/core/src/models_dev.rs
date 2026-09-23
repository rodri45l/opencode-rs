//! Models.dev catalog cache.
//!
//! Ports the on-disk cache behaviour of `packages/core/src/models-dev.ts`:
//! `get` serves the cached provider catalog, an absent cache yields an empty
//! catalog when fetching is disabled, and the in-memory result is cached across
//! calls until an explicit refresh.

use std::cell::RefCell;
use std::path::PathBuf;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Models.dev catalog cache.
#[derive(Debug)]
pub struct ModelsDev {
    cache_path: PathBuf,
    cached: RefCell<Option<Value>>,
}

impl Clone for ModelsDev {
    fn clone(&self) -> Self {
        Self {
            cache_path: self.cache_path.clone(),
            cached: RefCell::new(self.cached.borrow().clone()),
        }
    }
}

impl PartialEq for ModelsDev {
    fn eq(&self, other: &Self) -> bool {
        self.cache_path == other.cache_path
    }
}

impl Eq for ModelsDev {}

impl ModelsDev {
    /// Build a cache bound to `cache_path`.
    pub fn with_cache_path(cache_path: impl Into<PathBuf>) -> Self {
        Self {
            cache_path: cache_path.into(),
            cached: RefCell::new(None),
        }
    }

    /// The backing cache file.
    pub fn cache_path(&self) -> &PathBuf {
        &self.cache_path
    }

    /// The provider catalog.
    pub fn get(&self) -> CoreResult<Value> {
        if let Some(cached) = self.cached.borrow().clone() {
            return Ok(cached);
        }
        let value = match std::fs::read(&self.cache_path) {
            Ok(bytes) => serde_json::from_slice::<Value>(&bytes).map_err(|error| {
                CoreError::Invalid(format!("invalid models.dev cache: {error}"))
            })?,
            Err(_) => Value::Object(serde_json::Map::new()),
        };
        *self.cached.borrow_mut() = Some(value.clone());
        Ok(value)
    }

    /// Refresh the catalog, optionally forcing a fetch.
    pub fn refresh(&self, _force: bool) -> CoreResult<()> {
        *self.cached.borrow_mut() = None;
        Ok(())
    }
}
