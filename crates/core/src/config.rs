//! Configuration documents and v1 migration.
//!
//! Ports the observable behaviour of `packages/core/src/config.ts` and
//! `v1/config/migrate.ts`: priority-ordered documents expose the latest defined
//! scalar, v1 configuration is detected from any v1-only top-level key, and v1
//! documents migrate into the v2 shape.

use serde_json::{Map, Value};

use crate::path::AbsolutePath;
use crate::CoreResult;

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
    pub fn latest(entries: &[ConfigEntry], key: &str) -> CoreResult<Option<Value>> {
        for entry in entries.iter().rev() {
            if let ConfigEntry::Document(document) = entry {
                if let Some(value) = document.info.get(key) {
                    if !value.is_null() {
                        return Ok(Some(value.clone()));
                    }
                }
            }
        }
        Ok(None)
    }
}

/// Migrates v1 configuration into the v2 shape.
#[derive(Debug, Default)]
pub struct ConfigMigrateV1;

const V1_KEYS: &[&str] = &[
    "logLevel",
    "server",
    "command",
    "reference",
    "snapshot",
    "plugin",
    "autoshare",
    "disabled_providers",
    "enabled_providers",
    "small_model",
    "mode",
    "agent",
    "provider",
    "permission",
    "tools",
    "attachment",
    "layout",
];

impl ConfigMigrateV1 {
    /// Whether `value` is a v1 configuration document.
    pub fn is_v1(value: &Value) -> CoreResult<bool> {
        let Some(map) = value.as_object() else {
            return Ok(false);
        };
        Ok(map.keys().any(|key| V1_KEYS.contains(&key.as_str())))
    }

    /// Migrate a v1 configuration document to v2.
    pub fn migrate(value: &Value) -> CoreResult<Value> {
        let Some(map) = value.as_object() else {
            return Ok(value.clone());
        };
        let mut out = Map::new();
        if let Some(providers) = map.get("provider").and_then(Value::as_object) {
            let migrated: Map<String, Value> = providers
                .iter()
                .map(|(name, provider)| (name.clone(), migrate_provider(provider)))
                .collect();
            out.insert("providers".to_string(), Value::Object(migrated));
        }
        if let Some(commands) = map.get("command") {
            out.insert("commands".to_string(), commands.clone());
        }
        Ok(Value::Object(out))
    }
}

fn migrate_provider(provider: &Value) -> Value {
    let npm = provider.get("npm").and_then(Value::as_str);
    let options = provider.get("options").and_then(Value::as_object).cloned();
    let lowerer = npm
        .map(crate::provider_options::ConfigProviderOptionsV1::get)
        .unwrap_or_else(|| crate::provider_options::ConfigProviderOptionsV1::get(""));
    let lowered = options
        .as_ref()
        .map(|options| {
            lowerer
                .provider(&Value::Object(options.clone()))
                .unwrap_or(Value::Null)
        })
        .unwrap_or(Value::Null);
    let url = provider
        .get("api")
        .cloned()
        .or_else(|| lowered.get("url").cloned());
    let settings = lowered
        .get("settings")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let mut out = Map::new();
    if let Some(name) = provider.get("name") {
        out.insert("name".to_string(), name.clone());
    }
    if let Some(env) = provider.get("env") {
        out.insert("env".to_string(), env.clone());
    }
    if let Some(npm) = npm {
        let mut api = Map::new();
        api.insert("type".to_string(), Value::String("aisdk".into()));
        api.insert("package".to_string(), Value::String(npm.to_string()));
        if let Some(url) = url.filter(|value| !value.is_null()) {
            api.insert("url".to_string(), url);
        }
        api.insert("settings".to_string(), settings);
        out.insert("api".to_string(), Value::Object(api));
    }
    if options.is_some() {
        let mut request = Map::new();
        request.insert(
            "headers".to_string(),
            lowered
                .get("headers")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        );
        request.insert(
            "body".to_string(),
            lowered
                .get("body")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        );
        out.insert("request".to_string(), Value::Object(request));
    }
    Value::Object(out)
}
