//! Provider/model catalog and derived defaults.
//!
//! Ports the observable behaviour of `packages/core/src/catalog.ts`: transforms
//! build providers and models, provider/model `baseURL` is normalized into the
//! API url, provider and model request maps are merged with the model winning,
//! and `default`/`small` select a concrete model.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};

use crate::CoreResult;

/// Mutable catalog state handed to a transform.
#[derive(Debug, Default)]
pub struct CatalogEditor {
    providers: BTreeMap<String, Value>,
    models: BTreeMap<String, BTreeMap<String, Value>>,
    default_model: Option<(String, String)>,
}

impl CatalogEditor {
    /// Create an empty editor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update (creating if needed) a provider.
    pub fn provider_update<F>(&mut self, id: &str, update: F)
    where
        F: FnOnce(&mut Value),
    {
        let entry = self.providers.entry(id.to_string()).or_insert_with(|| {
            json!({
                "id": id,
                "name": id,
                "api": { "type": "native", "settings": {} },
                "request": { "headers": {}, "body": {} },
            })
        });
        update(entry);
        normalize_api(entry);
    }

    /// Update (creating if needed) a model under a provider.
    pub fn model_update<F>(&mut self, provider: &str, model: &str, update: F)
    where
        F: FnOnce(&mut Value),
    {
        let entry = self
            .models
            .entry(provider.to_string())
            .or_default()
            .entry(model.to_string())
            .or_insert_with(|| empty_model(provider, model));
        update(entry);
        if let Value::Object(map) = entry {
            map.insert("id".to_string(), json!(model));
            map.insert("providerID".to_string(), json!(provider));
        }
        normalize_api(entry);
    }

    /// Configure the default model for a provider.
    pub fn default_set(&mut self, provider: &str, model: &str) {
        self.default_model = Some((provider.to_string(), model.to_string()));
    }

    /// The default model selection recorded by transforms.
    pub fn default_model(&self) -> Option<&(String, String)> {
        self.default_model.as_ref()
    }
}

fn empty_model(provider: &str, model: &str) -> Value {
    json!({
        "id": model,
        "providerID": provider,
        "name": model,
        "api": { "id": model, "type": "native", "settings": {} },
        "capabilities": { "tools": false, "input": [], "output": [] },
        "request": { "headers": {}, "body": {} },
        "variants": [],
        "time": { "released": 0 },
        "cost": [],
        "status": "active",
        "enabled": true,
        "limit": { "context": 0, "output": 0 },
    })
}

fn normalize_api(item: &mut Value) {
    let Some(map) = item.as_object_mut() else {
        return;
    };
    let base_url = map
        .get("request")
        .and_then(Value::as_object)
        .and_then(|request| request.get("body"))
        .and_then(Value::as_object)
        .and_then(|body| body.get("baseURL"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let Some(base_url) = base_url else {
        return;
    };
    if let Some(api) = map.get_mut("api").and_then(Value::as_object_mut) {
        api.insert("url".to_string(), Value::String(base_url));
    }
    if let Some(body) = map
        .get_mut("request")
        .and_then(Value::as_object_mut)
        .and_then(|request| request.get_mut("body"))
        .and_then(Value::as_object_mut)
    {
        body.remove("baseURL");
    }
}

/// Provider/model catalog.
type CatalogTransform = Box<dyn Fn(&mut CatalogEditor)>;

#[derive(Default)]
pub struct Catalog {
    transforms: RefCell<Vec<CatalogTransform>>,
    providers: RefCell<BTreeMap<String, Value>>,
    models: RefCell<BTreeMap<String, BTreeMap<String, Value>>>,
    default_model: RefCell<Option<(String, String)>>,
}

impl std::fmt::Debug for Catalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Catalog").finish_non_exhaustive()
    }
}

impl Catalog {
    /// Create an empty catalog.
    pub fn new() -> CoreResult<Self> {
        Ok(Self::default())
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, transform: F) -> CoreResult<()>
    where
        F: Fn(&mut CatalogEditor) + 'static,
    {
        self.transforms.borrow_mut().push(Box::new(transform));
        self.rebuild()
    }

    /// Re-apply every registered transform.
    pub fn reload(&self) -> CoreResult<()> {
        self.rebuild()
    }

    fn rebuild(&self) -> CoreResult<()> {
        let mut editor = CatalogEditor::new();
        {
            for transform in self.transforms.borrow().iter() {
                transform(&mut editor);
            }
        }
        *self.providers.borrow_mut() = editor.providers;
        *self.models.borrow_mut() = editor.models;
        *self.default_model.borrow_mut() = editor.default_model;
        Ok(())
    }

    /// Look up a provider.
    pub fn provider_get(&self, id: &str) -> CoreResult<Option<Value>> {
        Ok(self.providers.borrow().get(id).cloned())
    }

    /// List all providers.
    pub fn provider_all(&self) -> CoreResult<Vec<Value>> {
        Ok(self.providers.borrow().values().cloned().collect())
    }

    /// List ids of providers that are currently available.
    pub fn provider_available(&self) -> CoreResult<Vec<String>> {
        Ok(self
            .providers
            .borrow()
            .iter()
            .filter(|(_, provider)| provider_available(provider))
            .map(|(id, _)| id.clone())
            .collect())
    }

    /// Look up a model under a provider.
    pub fn model_get(&self, provider: &str, model: &str) -> CoreResult<Option<Value>> {
        let models = self.models.borrow();
        let providers = self.providers.borrow();
        Ok(models
            .get(provider)
            .and_then(|entries| entries.get(model))
            .and_then(|model| {
                providers
                    .get(provider)
                    .map(|provider| project_model(model, provider))
            }))
    }

    /// List all models.
    pub fn model_all(&self) -> CoreResult<Vec<Value>> {
        let models = self.models.borrow();
        let providers = self.providers.borrow();
        let mut all: Vec<Value> = Vec::new();
        for (provider_id, entries) in models.iter() {
            if let Some(provider) = providers.get(provider_id) {
                for model in entries.values() {
                    all.push(project_model(model, provider));
                }
            }
        }
        all.sort_by(|left, right| {
            released(right)
                .partial_cmp(&released(left))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(all)
    }

    /// The default model, if one can be selected.
    pub fn model_default(&self) -> CoreResult<Option<Value>> {
        let default = self.default_model.borrow().clone();
        let available = self.provider_available()?;
        if let Some((provider_id, model_id)) = default {
            if available.iter().any(|id| id == &provider_id) {
                if let Some(model) = self.model_get(&provider_id, &model_id)? {
                    if model
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        return Ok(Some(model));
                    }
                }
            }
        }
        let available_models: Vec<Value> = self
            .model_all()?
            .into_iter()
            .filter(|model| {
                model
                    .get("providerID")
                    .and_then(Value::as_str)
                    .is_some_and(|provider| available.iter().any(|id| id == provider))
                    && model
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
            })
            .collect();
        Ok(available_models.into_iter().max_by(|left, right| {
            released(left)
                .partial_cmp(&released(right))
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
    }

    /// The preferred small model for a provider.
    pub fn model_small(&self, provider: &str) -> CoreResult<Option<Value>> {
        if provider == "azure" || provider == "azure-cognitive-services" {
            return Ok(None);
        }
        let models = self.models.borrow();
        let providers = self.providers.borrow();
        let Some(record) = models.get(provider) else {
            return Ok(None);
        };
        let Some(provider_info) = providers.get(provider) else {
            return Ok(None);
        };
        if provider == "opencode" {
            if let Some(nano) = record.get("gpt-5-nano") {
                if nano
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    && nano.get("status").and_then(Value::as_str) == Some("active")
                {
                    return Ok(Some(project_model(nano, provider_info)));
                }
            }
        }
        let now_days = now_millis() as f64 / (1000.0 * 60.0 * 60.0 * 24.0 * 30.0);
        let mut candidates: Vec<(&String, Value, f64, f64, bool)> = Vec::new();
        for (id, model) in record {
            if !model
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            if model.get("status").and_then(Value::as_str) != Some("active") {
                continue;
            }
            if !has_text(model, "input") || !has_text(model, "output") {
                continue;
            }
            let cost = first_cost(model);
            let age = now_days - released(model) / (1000.0 * 60.0 * 60.0 * 24.0 * 30.0);
            let small = is_small(id, model);
            candidates.push((id, model.clone(), cost, age, small));
        }
        candidates.retain(|(_, _, cost, _age, _)| *cost > 0.0);
        if candidates.is_empty() {
            return Ok(None);
        }
        let small_only: Vec<_> = candidates
            .iter()
            .filter(|(_, _, _, _, small)| *small)
            .cloned()
            .collect();
        let pool = if small_only.is_empty() {
            candidates
        } else {
            small_only
        };
        let max_cost = pool
            .iter()
            .map(|(_, _, cost, _, _)| *cost)
            .fold(0.01_f64, f64::max);
        let max_age = pool
            .iter()
            .map(|(_, _, _, age, _)| *age)
            .fold(0.01_f64, f64::max);
        let pick = pool
            .into_iter()
            .min_by(|(_, _, cost_a, age_a, _), (_, _, cost_b, age_b, _)| {
                let score_a = (cost_a / max_cost) * 0.8 + (age_a / max_age) * 0.2;
                let score_b = (cost_b / max_cost) * 0.8 + (age_b / max_age) * 0.2;
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        Ok(pick.map(|(_, model, _, _, _)| project_model(&model, provider_info)))
    }
}

fn provider_available(provider: &Value) -> bool {
    if provider.get("disabled").and_then(Value::as_bool) == Some(true) {
        return false;
    }
    if let Some(api_key) = provider
        .get("request")
        .and_then(Value::as_object)
        .and_then(|request| request.get("body"))
        .and_then(Value::as_object)
        .and_then(|body| body.get("apiKey"))
        .and_then(Value::as_str)
    {
        if !api_key.is_empty() {
            return true;
        }
    }
    provider.get("integrationID").is_none()
}

fn project_model(model: &Value, provider: &Value) -> Value {
    let provider_api = provider.get("api").cloned().unwrap_or(Value::Null);
    let model_api = model.get("api").cloned().unwrap_or(Value::Null);
    let api = project_api(&model_api, &provider_api, model);
    let provider_request = provider
        .get("request")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let model_request = model.get("request").cloned().unwrap_or_else(|| json!({}));
    let headers = merge_objects(
        provider_request.get("headers"),
        model_request.get("headers"),
    );
    let body = merge_objects(provider_request.get("body"), model_request.get("body"));
    let mut projected = model.clone();
    if let Value::Object(map) = &mut projected {
        map.insert("api".to_string(), api);
        map.insert(
            "request".to_string(),
            json!({
                "headers": headers,
                "body": body,
                "variant": model_request.get("variant").cloned().unwrap_or(Value::Null),
            }),
        );
    }
    projected
}

fn project_api(model_api: &Value, provider_api: &Value, model: &Value) -> Value {
    let model_type = model_api.get("type").and_then(Value::as_str).unwrap_or("");
    let provider_type = provider_api
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("");
    let model_url = model_api.get("url").and_then(Value::as_str);
    let model_settings_empty = model_api
        .get("settings")
        .and_then(Value::as_object)
        .map(|settings| settings.is_empty())
        .unwrap_or(true);
    if model_type == "native" && model_url.is_none() && model_settings_empty {
        let mut merged = provider_api.clone();
        if let Some(map) = merged.as_object_mut() {
            let id = model
                .get("api")
                .and_then(|api| api.get("id"))
                .or_else(|| model.get("id"))
                .cloned()
                .unwrap_or(Value::Null);
            map.insert("id".to_string(), id);
        }
        return merged;
    }
    if model_type == "aisdk" && provider_type == "aisdk" {
        let mut merged = model_api.clone();
        if let Some(map) = merged.as_object_mut() {
            if model_url.is_none() {
                if let Some(url) = provider_api.get("url") {
                    map.insert("url".to_string(), url.clone());
                }
            }
            let settings = merge_objects(provider_api.get("settings"), model_api.get("settings"));
            map.insert("settings".to_string(), settings);
        }
        return merged;
    }
    model_api.clone()
}

fn merge_objects(left: Option<&Value>, right: Option<&Value>) -> Value {
    let mut merged = Map::new();
    if let Some(Value::Object(left)) = left {
        for (key, value) in left {
            merged.insert(key.clone(), value.clone());
        }
    }
    if let Some(Value::Object(right)) = right {
        for (key, value) in right {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn released(value: &Value) -> f64 {
    value
        .get("time")
        .and_then(Value::as_object)
        .and_then(|time| time.get("released"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}

fn has_text(model: &Value, direction: &str) -> bool {
    model
        .get("capabilities")
        .and_then(Value::as_object)
        .and_then(|capabilities| capabilities.get(direction))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .any(|item| item.as_str().is_some_and(|text| text.starts_with("text")))
        })
        .unwrap_or(false)
}

fn first_cost(model: &Value) -> f64 {
    model
        .get("cost")
        .and_then(Value::as_array)
        .and_then(|costs| costs.first())
        .map(|cost| {
            cost.get("input").and_then(Value::as_f64).unwrap_or(0.0)
                + cost.get("output").and_then(Value::as_f64).unwrap_or(0.0)
        })
        .unwrap_or(999.0)
}

fn is_small(id: &str, model: &Value) -> bool {
    let family = model.get("family").and_then(Value::as_str).unwrap_or("");
    let name = model.get("name").and_then(Value::as_str).unwrap_or("");
    let haystack = format!("{id} {family} {name}").to_lowercase();
    ["nano", "flash", "lite", "mini", "haiku", "small", "fast"]
        .iter()
        .any(|keyword| haystack.contains(keyword))
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
