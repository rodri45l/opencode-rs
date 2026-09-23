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
    pub fn parse_specs(plugins: &Value) -> CoreResult<Vec<PluginSpec>> {
        let items = match plugins.as_array() {
            Some(items) => items,
            None => return Ok(Vec::new()),
        };
        let mut specs = Vec::new();
        for item in items {
            if let Some(package) = item.as_str() {
                specs.push(PluginSpec {
                    package: package.to_string(),
                    options: BTreeMap::new(),
                });
                continue;
            }
            let package = item
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let mut options = BTreeMap::new();
            if let Some(map) = item.get("options").and_then(Value::as_object) {
                for (key, value) in map {
                    let rendered = match value {
                        Value::String(text) => text.clone(),
                        other => other.to_string(),
                    };
                    options.insert(key.clone(), rendered);
                }
            }
            specs.push(PluginSpec { package, options });
        }
        Ok(specs)
    }

    /// Whether a package reference is a relative file path.
    pub fn is_relative_reference(package: &str) -> CoreResult<bool> {
        Ok(package.starts_with("./") || package.starts_with("../") || package.starts_with('/'))
    }

    /// Whether a package reference is an npm specifier.
    pub fn is_npm_spec(package: &str) -> CoreResult<bool> {
        if package.is_empty() || Self::is_relative_reference(package)? {
            return Ok(false);
        }
        if package.starts_with("file:") || package.starts_with("git+") {
            return Ok(false);
        }
        Ok(true)
    }

    /// Whether a file name is a loadable plugin file.
    pub fn is_plugin_file(name: &str) -> CoreResult<bool> {
        Ok(name.ends_with(".ts") || name.ends_with(".mts") || name.ends_with(".js"))
    }

    /// Filter out entries that cannot be loaded (empty package), preserving order.
    pub fn loadable_specs(specs: Vec<PluginSpec>) -> CoreResult<Vec<PluginSpec>> {
        Ok(specs
            .into_iter()
            .filter(|spec| !spec.package.trim().is_empty())
            .collect())
    }
}

/// Error for a plugin reference that cannot be classified.
pub fn invalid_reference(package: &str) -> CoreError {
    CoreError::Invalid(format!("invalid plugin reference: {package}"))
}
