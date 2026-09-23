//! External config plugin loading (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/config/plugin/external.ts`:
//! config plugin entries may be a bare package string or an object with
//! `package` and `options`; relative file references and npm specifiers are
//! distinguished; and plugin files are discovered by extension in config
//! directories. The `Config`/`Npm`/`PluginHost`/`FSUtil` service wiring and the
//! async module loading are dropped; the pure classification remains.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A normalized config plugin entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSpec {
    /// The package or file reference.
    pub package: String,
    /// String options passed to the plugin.
    pub options: BTreeMap<String, String>,
}

/// The external config plugin loader.
#[derive(Debug, Default)]
pub struct ConfigExternalPlugin;

impl ConfigExternalPlugin {
    /// Parse a config `plugins` array into normalized specs.
    pub fn parse_specs(_plugins: &Value) -> CoreResult<Vec<PluginSpec>> {
        Err(CoreError::NotImplemented(
            "config_plugin::ConfigExternalPlugin::parse_specs",
        ))
    }

    /// Whether a package reference is a relative file path.
    pub fn is_relative_reference(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "config_plugin::ConfigExternalPlugin::is_relative_reference",
        ))
    }

    /// Whether a package reference is an npm specifier.
    pub fn is_npm_spec(_package: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "config_plugin::ConfigExternalPlugin::is_npm_spec",
        ))
    }

    /// Whether a file name is a loadable plugin file.
    pub fn is_plugin_file(_name: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "config_plugin::ConfigExternalPlugin::is_plugin_file",
        ))
    }

    /// Filter out entries that cannot be loaded (empty package), preserving order.
    pub fn loadable_specs(_specs: Vec<PluginSpec>) -> CoreResult<Vec<PluginSpec>> {
        Err(CoreError::NotImplemented(
            "config_plugin::ConfigExternalPlugin::loadable_specs",
        ))
    }
}
