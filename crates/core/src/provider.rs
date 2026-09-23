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

use crate::{CoreError, CoreResult};

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
    pub fn build(_documents: &[Value]) -> CoreResult<CatalogSnapshot> {
        Err(CoreError::NotImplemented(
            "provider::ConfigProviderPlugin::build",
        ))
    }
}
