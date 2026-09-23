//! Location service map.
//!
//! Ports the observable behaviour of
//! `packages/core/src/location-services.ts`: a location reference constructed
//! directly or decoded from plain data shares one cached service context, and
//! location state (configured providers) is isolated per directory while the
//! shared location policy can still deny a provider by resource.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A location reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocationRef {
    /// Canonical directory.
    pub directory: String,
    /// Optional workspace id.
    pub workspace_id: Option<String>,
}

impl LocationRef {
    /// Construct a reference for a directory.
    pub fn new(directory: impl Into<String>) -> Self {
        Self {
            directory: directory.into(),
            workspace_id: None,
        }
    }

    /// Decode a reference from plain JSON.
    pub fn decode(_value: &Value) -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "location_layer::LocationRef::decode",
        ))
    }
}

/// Cached per-location services.
#[derive(Debug, Default)]
pub struct LocationServiceMap {
    configured: BTreeMap<String, Vec<String>>,
    policy: Vec<Value>,
}

impl LocationServiceMap {
    /// Create an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached context identity for a reference.
    pub fn context_effect(&self, _reference: &LocationRef) -> CoreResult<u64> {
        let _ = (&self.configured, &self.policy);
        Err(CoreError::NotImplemented(
            "location_layer::LocationServiceMap::context_effect",
        ))
    }

    /// Configure providers for a location.
    pub fn configure(
        &mut self,
        _reference: &LocationRef,
        _providers: Vec<String>,
    ) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "location_layer::LocationServiceMap::configure",
        ))
    }

    /// Apply the shared policy to a location's configured providers.
    pub fn provider_ids(&self, _reference: &LocationRef) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented(
            "location_layer::LocationServiceMap::provider_ids",
        ))
    }

    /// Record a policy rule (`action`/`resource`) shared across locations.
    pub fn add_policy(&mut self, _rule: Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "location_layer::LocationServiceMap::add_policy",
        ))
    }
}
