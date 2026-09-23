//! Models.dev provider plugin.
//!
//! Ports the observable behaviour of `packages/core/src/plugin/models-dev.ts`:
//! experimental `modes` are projected as separate models (not variants), their
//! cost list is normalized (mode cost, context tiers, then the legacy
//! `context_over_200k` tier), and providers with environment variables register a
//! key method plus an env method.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

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
    pub fn project_models(_provider: &ModelsDevProvider) -> CoreResult<Vec<ProjectedModel>> {
        Err(CoreError::NotImplemented(
            "models_dev_plugin::ModelsDevPlugin::project_models",
        ))
    }

    /// Register integration methods for a provider.
    pub fn integration_methods(_provider: &ModelsDevProvider) -> CoreResult<Vec<Value>> {
        Err(CoreError::NotImplemented(
            "models_dev_plugin::ModelsDevPlugin::integration_methods",
        ))
    }
}
