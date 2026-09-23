//! Project `.npmrc` loading.
//!
//! Ports the observable behaviour of `packages/core/src/npm-config.ts`: read a
//! project `.npmrc` into a normalized map (camelCase booleans, flattened list
//! options, scoped registry keys preserved) and resolve the registry without a
//! trailing slash.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::{CoreError, CoreResult};

/// Project npm configuration helpers.
#[derive(Debug, Default)]
pub struct NpmConfig;

impl NpmConfig {
    /// Load the normalized npm configuration for `directory`.
    pub fn load(directory: &str) -> CoreResult<Value> {
        let file = Path::new(directory).join(".npmrc");
        let contents = match std::fs::read_to_string(&file) {
            Ok(contents) => contents,
            Err(_) => return Ok(Value::Object(Map::new())),
        };
        let mut config = Map::new();
        for raw in contents.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let (key, value) = match line.split_once('=') {
                Some((key, value)) => (key.trim(), value.trim()),
                None => continue,
            };
            if key.is_empty() {
                continue;
            }
            if let Some(base) = key.strip_suffix("[]") {
                let entry = config
                    .entry(normalize_key(base))
                    .or_insert_with(|| Value::Array(Vec::new()));
                match entry {
                    Value::Array(items) => items.push(json!(value)),
                    other => {
                        *other = json!([value]);
                    }
                }
                continue;
            }
            let normalized = normalize_key(key);
            if value.eq_ignore_ascii_case("true") {
                config.insert(normalized, json!(true));
            } else if value.eq_ignore_ascii_case("false") {
                config.insert(normalized, json!(false));
            } else {
                config.insert(normalized, json!(value));
            }
        }
        Ok(Value::Object(config))
    }

    /// Resolve the configured registry for `directory`, without a trailing slash.
    pub fn registry(directory: &str) -> CoreResult<String> {
        let config = Self::load(directory)?;
        let registry = config
            .get("registry")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("https://registry.npmjs.org");
        Ok(registry.strip_suffix('/').unwrap_or(registry).to_string())
    }

    /// Whether a config map requests a specific boolean.
    pub fn boolean(value: &Value, key: &str) -> CoreResult<bool> {
        Ok(value.get(key).and_then(Value::as_bool).unwrap_or(false))
    }
}

/// Error for an unreadable config file.
pub fn unreadable(directory: &str) -> CoreError {
    CoreError::FileSystem(format!("cannot read npm config in {directory}"))
}

/// Normalize an npm option key the way `@npmcli/config` flattens it: known
/// kebab-case keys become camelCase, scoped registry keys are preserved.
fn normalize_key(key: &str) -> String {
    if key.starts_with('@') && key.contains(':') {
        return key.to_string();
    }
    let mut out = String::new();
    let mut upper = false;
    for ch in key.chars() {
        if ch == '-' || ch == '_' {
            upper = true;
            continue;
        }
        if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    // Nested option keys (`a.b`) keep their dots and camelCase each segment.
    out
}
