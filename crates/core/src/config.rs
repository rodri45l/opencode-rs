//! Configuration documents and v1 migration.
//!
//! Ports the observable behaviour of `packages/core/src/config.ts` and
//! `v1/config/migrate.ts`: priority-ordered documents expose the latest defined
//! scalar, v1 configuration is detected from any v1-only top-level key, and v1
//! documents migrate into the v2 shape.

use serde_json::Value;

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A loaded configuration document.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigDocument {
    /// Decoded document contents.
    pub info: Value,
}

impl ConfigDocument {
    /// Build a document from decoded contents.
    pub fn new(info: Value) -> Self {
        Self { info }
    }
}

/// A priority-ordered configuration entry.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigEntry {
    /// A file document.
    Document(ConfigDocument),
    /// A directory that contributed documents.
    Directory(AbsolutePath),
}

/// Configuration helpers.
#[derive(Debug, Default)]
pub struct Config;

impl Config {
    /// The latest defined scalar `key` across priority-ordered entries.
    pub fn latest(_entries: &[ConfigEntry], _key: &str) -> CoreResult<Option<Value>> {
        Err(CoreError::NotImplemented("config::Config::latest"))
    }
}

/// Migrates v1 configuration into the v2 shape.
#[derive(Debug, Default)]
pub struct ConfigMigrateV1;

impl ConfigMigrateV1 {
    /// Whether `value` is a v1 configuration document.
    pub fn is_v1(_value: &Value) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("config::ConfigMigrateV1::is_v1"))
    }

    /// Migrate a v1 configuration document to v2.
    pub fn migrate(_value: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "config::ConfigMigrateV1::migrate",
        ))
    }
}
