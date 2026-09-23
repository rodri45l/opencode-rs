//! Config provider plugin.
//!
//! Ports the observable behaviour of
//! `packages/core/src/config/plugin/provider.ts`: configured provider documents
//! are loaded in order, later documents override earlier ones, request
//! headers/body maps merge (`shared` resolved by the last document), model
//! `api`/`limit`/`cost` merge, variants merge by id, and the last `model`
//! declaration selects the default.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::CoreResult;

/// A merged provider variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Variant id.
    pub id: String,
    /// Variant request headers.
    pub headers: Value,
    /// Variant request body.
    pub body: Value,
}

/// A merged model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// Model id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Whether the model is enabled.
    pub enabled: bool,
    /// API descriptor.
    pub api: Value,
    /// Token limits.
    pub limit: Value,
    /// Cost entries.
    pub cost: Value,
    /// Merged request envelope.
    pub request: Value,
    /// Variants in declaration order.
    pub variants: Vec<Variant>,
}

/// A merged provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    /// Provider id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Whether the provider is disabled.
    pub disabled: bool,
    /// API descriptor.
    pub api: Value,
    /// Merged request envelope.
    pub request: Value,
    /// Models by id.
    pub models: BTreeMap<String, Model>,
}

/// A merged catalog snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogSnapshot {
    /// The default model reference (`provider/model`).
    pub default_model: Option<String>,
    /// Providers by id.
    pub providers: BTreeMap<String, Provider>,
}

/// The config provider plugin.
#[derive(Debug, Default)]
pub struct ConfigProviderPlugin;

impl ConfigProviderPlugin {
    /// Load and merge configured provider documents.
    pub fn build(documents: &[Value]) -> CoreResult<CatalogSnapshot> {
        let mut snapshot = CatalogSnapshot::default();
        for document in documents {
            if let Some(model) = document.get("model").and_then(Value::as_str) {
                snapshot.default_model = Some(model.to_string());
            }
            let Some(providers) = document.get("providers").and_then(Value::as_object) else {
                continue;
            };
            for (id, item) in providers {
                let provider = snapshot
                    .providers
                    .entry(id.clone())
                    .or_insert_with(|| empty_provider(id));
                if let Some(name) = item.get("name").and_then(Value::as_str) {
                    provider.name = name.to_string();
                }
                if let Some(api) = item.get("api") {
                    provider.api = api.clone();
                }
                if let Some(request) = item.get("request") {
                    merge_into(&mut provider.request, "headers", request.get("headers"));
                    merge_into(&mut provider.request, "body", request.get("body"));
                }
                let Some(models) = item.get("models").and_then(Value::as_object) else {
                    continue;
                };
                for (model_id, config) in models {
                    let model = provider
                        .models
                        .entry(model_id.clone())
                        .or_insert_with(|| empty_model(model_id));
                    apply_model(model, config);
                }
            }
        }
        Ok(snapshot)
    }
}

fn empty_provider(id: &str) -> Provider {
    Provider {
        id: id.to_string(),
        name: id.to_string(),
        disabled: false,
        api: serde_json::json!({ "type": "native", "settings": {} }),
        request: serde_json::json!({ "headers": {}, "body": {} }),
        models: BTreeMap::new(),
    }
}

fn empty_model(id: &str) -> Model {
    Model {
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        api: serde_json::json!({ "id": id, "type": "native", "settings": {} }),
        limit: serde_json::json!({ "context": 0, "output": 0 }),
        cost: Value::Array(Vec::new()),
        request: serde_json::json!({ "headers": {}, "body": {} }),
        variants: Vec::new(),
    }
}

fn apply_model(model: &mut Model, config: &Value) {
    if let Some(family) = config.get("family").and_then(Value::as_str) {
        model.api = ensure(model.api.clone(), "family", serde_json::json!(family));
    }
    if let Some(name) = config.get("name").and_then(Value::as_str) {
        model.name = name.to_string();
    }
    if let Some(api) = config.get("api").and_then(Value::as_object) {
        if let Value::Object(current) = &mut model.api {
            for (key, value) in api {
                current.insert(key.clone(), value.clone());
            }
        } else {
            model.api = Value::Object(api.clone());
        }
    }
    if let Some(request) = config.get("request") {
        merge_into(&mut model.request, "headers", request.get("headers"));
        merge_into(&mut model.request, "body", request.get("body"));
        if let Some(variant) = request.get("variant") {
            set_key(&mut model.request, "variant", variant.clone());
        }
    }
    if let Some(variants) = config.get("variants").and_then(Value::as_array) {
        for variant in variants {
            let Some(variant_id) = variant.get("id").and_then(Value::as_str) else {
                continue;
            };
            let slot = match model.variants.iter_mut().find(|item| item.id == variant_id) {
                Some(existing) => existing,
                None => {
                    model.variants.push(Variant {
                        id: variant_id.to_string(),
                        headers: serde_json::json!({}),
                        body: serde_json::json!({}),
                    });
                    model.variants.last_mut().expect("just pushed")
                }
            };
            merge_into(&mut slot.headers, "self", variant.get("headers"));
            merge_into(&mut slot.body, "self", variant.get("body"));
        }
    }
    if let Some(cost) = config.get("cost") {
        let entries: Vec<&Value> = match cost {
            Value::Array(items) => items.iter().collect(),
            other => vec![other],
        };
        model.cost = Value::Array(
            entries
                .into_iter()
                .map(|entry| {
                    let mut out = serde_json::Map::new();
                    if let Some(tier) = entry.get("tier").filter(|tier| !tier.is_null()) {
                        out.insert("tier".to_string(), tier.clone());
                    }
                    if let Some(input) = entry.get("input") {
                        out.insert("input".to_string(), input.clone());
                    }
                    if let Some(output) = entry.get("output") {
                        out.insert("output".to_string(), output.clone());
                    }
                    if let Some(cache) = entry.get("cache").filter(|cache| cache.is_object()) {
                        out.insert(
                            "cache".to_string(),
                            serde_json::json!({
                                "read": cache.get("read").cloned().unwrap_or(serde_json::json!(0)),
                                "write": cache.get("write").cloned().unwrap_or(serde_json::json!(0)),
                            }),
                        );
                    }
                    Value::Object(out)
                })
                .collect(),
        );
    }
    if let Some(disabled) = config.get("disabled").and_then(Value::as_bool) {
        model.enabled = !disabled;
    }
    if let Some(limit) = config.get("limit") {
        merge_into(&mut model.limit, "self", Some(limit));
    }
}

fn set_key(target: &mut Value, key: &str, value: Value) {
    if let Value::Object(map) = target {
        map.insert(key.to_string(), value);
    }
}

fn ensure(mut target: Value, key: &str, value: Value) -> Value {
    set_key(&mut target, key, value);
    target
}

fn merge_into(target: &mut Value, key: &str, source: Option<&Value>) {
    let Some(source) = source.and_then(Value::as_object) else {
        return;
    };
    let slot = if key == "self" {
        target
    } else {
        if !target.is_object() {
            *target = serde_json::json!({});
        }
        if !target.get(key).is_some_and(Value::is_object) {
            set_key(target, key, serde_json::json!({}));
        }
        match target.get_mut(key) {
            Some(value) => value,
            None => return,
        }
    };
    if let Value::Object(map) = slot {
        for (entry_key, entry_value) in source {
            map.insert(entry_key.clone(), entry_value.clone());
        }
    }
}
