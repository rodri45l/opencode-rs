//! Provider/model catalog and derived defaults.
//!
//! Ports the observable behaviour of `packages/core/src/catalog.ts`: transforms
//! build providers and models, provider/model `baseURL` is normalized into the
//! API url, provider and model request maps are merged with the model winning,
//! and `default`/`small` select a concrete model.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

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
        let entry = self.providers.entry(id.to_string()).or_insert_with(
            || serde_json::json!({ "id": id, "request": { "headers": {}, "body": {} } }),
        );
        update(entry);
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
            .or_insert_with(|| {
                serde_json::json!({
                    "id": model,
                    "providerID": provider,
                    "request": { "headers": {}, "body": {} },
                })
            });
        update(entry);
    }

    /// Configure the default model for a provider.
    pub fn default_set(&mut self, provider: &str, model: &str) {
        self.default_model = Some((provider.to_string(), model.to_string()));
    }
}

/// Provider/model catalog.
#[derive(Debug, Default)]
pub struct Catalog;

impl Catalog {
    /// Create an empty catalog.
    pub fn new() -> CoreResult<Self> {
        Err(CoreError::NotImplemented("catalog::Catalog::new"))
    }

    /// Register a replayable transform.
    pub fn transform<F>(&self, _transform: F) -> CoreResult<()>
    where
        F: Fn(&mut CatalogEditor) + 'static,
    {
        Err(CoreError::NotImplemented("catalog::Catalog::transform"))
    }

    /// Re-apply every registered transform.
    pub fn reload(&self) -> CoreResult<()> {
        Err(CoreError::NotImplemented("catalog::Catalog::reload"))
    }

    /// Look up a provider.
    pub fn provider_get(&self, _id: &str) -> CoreResult<Option<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::provider_get"))
    }

    /// List all providers.
    pub fn provider_all(&self) -> CoreResult<Vec<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::provider_all"))
    }

    /// List ids of providers that are currently available.
    pub fn provider_available(&self) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented(
            "catalog::Catalog::provider_available",
        ))
    }

    /// Look up a model under a provider.
    pub fn model_get(&self, _provider: &str, _model: &str) -> CoreResult<Option<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::model_get"))
    }

    /// List all models.
    pub fn model_all(&self) -> CoreResult<Vec<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::model_all"))
    }

    /// The default model, if one can be selected.
    pub fn model_default(&self) -> CoreResult<Option<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::model_default"))
    }

    /// The preferred small model for a provider.
    pub fn model_small(&self, _provider: &str) -> CoreResult<Option<Value>> {
        Err(CoreError::NotImplemented("catalog::Catalog::model_small"))
    }
}
