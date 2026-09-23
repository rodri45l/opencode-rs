//! Models.dev provider plugin.
//!
//! Ports the observable behaviour of `packages/core/src/plugin/models-dev.ts`:
//! experimental `modes` are projected as separate models (not variants), their
//! cost list is normalized (mode cost, context tiers, then the legacy
//! `context_over_200k` tier), and providers with environment variables register a
//! key method plus an env method.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::CoreResult;

/// A models.dev model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelsDevModel {
    /// Model id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Provider-relative cost descriptor.
    pub cost: Value,
    /// Token limits.
    pub limit: Value,
    /// Experimental modes keyed by mode id.
    pub modes: BTreeMap<String, Value>,
}

/// A models.dev provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelsDevProvider {
    /// Provider id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Environment variable names.
    pub env: Vec<String>,
    /// npm package name.
    pub npm: Option<String>,
    /// API base URL.
    pub api: Option<String>,
    /// Models by id.
    pub models: BTreeMap<String, ModelsDevModel>,
}

/// A model projected into the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedModel {
    /// Catalog model id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// API model id (the source model id).
    pub api_id: String,
    /// Merged request envelope.
    pub request: Value,
    /// Normalized cost entries.
    pub cost: Vec<Value>,
    /// Variants (always empty for projected models).
    pub variants: Vec<Value>,
}

/// The models.dev provider plugin.
#[derive(Debug, Default)]
pub struct ModelsDevPlugin;

impl ModelsDevPlugin {
    /// Project a provider's models, turning modes into separate models.
    pub fn project_models(provider: &ModelsDevProvider) -> CoreResult<Vec<ProjectedModel>> {
        let mut projected = Vec::new();
        for (id, model) in &provider.models {
            projected.push(ProjectedModel {
                id: id.clone(),
                name: model.name.clone(),
                api_id: model.id.clone(),
                request: serde_json::json!({ "headers": {}, "body": {} }),
                cost: normalize_cost(&model.cost),
                variants: Vec::new(),
            });
            for (mode, options) in &model.modes {
                let name = format!("{} {}{}", model.name, uppercase_first(mode), "");
                let request = options
                    .get("provider")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({ "headers": {}, "body": {} }));
                let request = serde_json::json!({
                    "headers": request.get("headers").cloned().unwrap_or_else(|| serde_json::json!({})),
                    "body": request.get("body").cloned().unwrap_or_else(|| serde_json::json!({})),
                });
                let cost = match options.get("cost") {
                    Some(override_cost) => merge_cost(&normalize_cost(&model.cost), override_cost),
                    None => normalize_cost(&model.cost),
                };
                projected.push(ProjectedModel {
                    id: format!("{id}-{mode}"),
                    name,
                    api_id: model.id.clone(),
                    request,
                    cost,
                    variants: Vec::new(),
                });
            }
        }
        Ok(projected)
    }

    /// Register integration methods for a provider.
    pub fn integration_methods(provider: &ModelsDevProvider) -> CoreResult<Vec<Value>> {
        if provider.env.is_empty() {
            return Ok(Vec::new());
        }
        Ok(vec![
            serde_json::json!({ "type": "key" }),
            serde_json::json!({ "type": "env", "names": provider.env }),
        ])
    }
}

fn uppercase_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn number_or(value: Option<&Value>, default: i64) -> Value {
    match value {
        Some(value) if value.is_number() => value.clone(),
        _ => serde_json::json!(default),
    }
}

fn normalize_cost(cost: &Value) -> Vec<Value> {
    let entry = serde_json::json!({
        "input": number_or(cost.get("input"), 0),
        "output": number_or(cost.get("output"), 0),
        "cache": {
            "read": number_or(cost.get("cache_read"), 0),
            "write": number_or(cost.get("cache_write"), 0),
        },
    });
    let mut entries = vec![entry];
    if let Some(tiers) = cost.get("tiers").and_then(Value::as_array) {
        for tier in tiers {
            entries.push(serde_json::json!({
                "tier": tier.get("tier").cloned().unwrap_or(Value::Null),
                "input": number_or(tier.get("input"), 0),
                "output": number_or(tier.get("output"), 0),
                "cache": {
                    "read": number_or(tier.get("cache_read"), 0),
                    "write": number_or(tier.get("cache_write"), 0),
                },
            }));
        }
    }
    if let Some(over) = cost.get("context_over_200k") {
        entries.push(serde_json::json!({
            "tier": { "type": "context", "size": 200_000 },
            "input": number_or(over.get("input"), 0),
            "output": number_or(over.get("output"), 0),
            "cache": {
                "read": number_or(over.get("cache_read"), 0),
                "write": number_or(over.get("cache_write"), 0),
            },
        }));
    }
    entries
}

fn merge_cost(base: &[Value], override_cost: &Value) -> Vec<Value> {
    let next = normalize_cost(override_cost);
    let base_default = base.first().cloned();
    let next_default = next.first().cloned();
    let mut out = Vec::new();
    match (base_default, next_default) {
        (Some(base_default), Some(next_default)) => {
            out.push(merge_entry(&base_default, &next_default));
        }
        (None, Some(next_default)) => out.push(next_default),
        _ => {}
    }
    let mut tiers: Vec<Value> = base.iter().skip(1).cloned().collect();
    for item in next.iter().skip(1) {
        let key = tier_key(item);
        match tiers.iter_mut().find(|existing| tier_key(existing) == key) {
            Some(existing) => *existing = merge_entry(existing, item),
            None => tiers.push(item.clone()),
        }
    }
    out.extend(tiers);
    out
}

fn merge_entry(left: &Value, right: &Value) -> Value {
    let mut merged = left.clone();
    if let (Some(left_map), Some(right_map)) = (merged.as_object_mut(), right.as_object()) {
        for (key, value) in right_map {
            if key == "cache" {
                if let (Some(left_cache), Some(right_cache)) = (
                    left_map.get_mut("cache").and_then(Value::as_object_mut),
                    value.as_object(),
                ) {
                    for (cache_key, cache_value) in right_cache {
                        left_cache.insert(cache_key.clone(), cache_value.clone());
                    }
                }
                continue;
            }
            if key == "tier" && value.is_null() {
                continue;
            }
            left_map.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn tier_key(item: &Value) -> String {
    match item.get("tier") {
        Some(tier) if !tier.is_null() => format!(
            "{}:{}",
            tier.get("type").and_then(Value::as_str).unwrap_or("base"),
            tier.get("size").and_then(Value::as_i64).unwrap_or(0)
        ),
        _ => "base:0".to_string(),
    }
}
