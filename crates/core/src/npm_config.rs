//! Project `.npmrc` loading.
//!
//! Ports the observable behaviour of `packages/core/src/npm-config.ts`: read a
//! project `.npmrc` into a normalized map (camelCase booleans, flattened list
//! options, scoped registry keys preserved) and resolve the registry without a
//! trailing slash.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Project npm configuration helpers.
#[derive(Debug, Default)]
pub struct NpmConfig;

impl NpmConfig {
    /// Load the normalized npm configuration for `directory`.
    pub fn load(_directory: &str) -> CoreResult<Value> {
        Err(CoreError::NotImplemented("npm_config::NpmConfig::load"))
    }

    /// Resolve the configured registry for `directory`, without a trailing slash.
    pub fn registry(_directory: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented("npm_config::NpmConfig::registry"))
    }
}
