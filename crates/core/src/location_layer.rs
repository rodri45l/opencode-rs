//! Location service map.
//!
//! Ports the observable behaviour of
//! `packages/core/src/location-services.ts`: a location reference constructed
//! directly or decoded from plain data shares one cached service context, and
//! location state (configured providers and policy) is isolated per directory
//! while a policy can deny a provider by resource.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

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
    pub fn decode(value: &Value) -> CoreResult<Self> {
        let directory = value
            .get("directory")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("location ref missing directory".into()))?;
        let workspace_id = value
            .get("workspaceID")
            .or_else(|| value.get("workspaceId"))
            .or_else(|| value.get("workspace_id"))
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(Self {
            directory: directory.to_string(),
            workspace_id,
        })
    }
}

/// Cached per-location services.
#[derive(Debug, Default)]
pub struct LocationServiceMap {
    configured: BTreeMap<String, Vec<String>>,
    policies: BTreeMap<String, Vec<Value>>,
}

impl LocationServiceMap {
    /// Create an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// The cached context identity for a reference.
    pub fn context_effect(&self, reference: &LocationRef) -> CoreResult<u64> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        reference.hash(&mut hasher);
        Ok(hasher.finish())
    }

    /// Configure providers for a location.
    pub fn configure(&mut self, reference: &LocationRef, providers: Vec<String>) -> CoreResult<()> {
        self.configured
            .insert(reference.directory.clone(), providers);
        Ok(())
    }

    /// Apply the location's policy to its configured providers.
    pub fn provider_ids(&self, reference: &LocationRef) -> CoreResult<Vec<String>> {
        let providers = self
            .configured
            .get(&reference.directory)
            .cloned()
            .unwrap_or_default();
        let policies = self
            .policies
            .get(&reference.directory)
            .cloned()
            .unwrap_or_default();
        Ok(providers
            .into_iter()
            .filter(|provider| !denied(provider, &policies))
            .collect())
    }

    /// Record a policy rule (`effect`/`action`/`resource`) for a location.
    pub fn add_policy(&mut self, reference: &LocationRef, rule: Value) -> CoreResult<()> {
        self.policies
            .entry(reference.directory.clone())
            .or_default()
            .push(rule);
        Ok(())
    }
}

fn denied(provider: &str, policies: &[Value]) -> bool {
    let mut effect = "allow";
    for policy in policies {
        if policy.get("action").and_then(Value::as_str) != Some("provider.use") {
            continue;
        }
        let resource = policy.get("resource").and_then(Value::as_str).unwrap_or("");
        if resource == provider || resource == "*" {
            effect = policy
                .get("effect")
                .and_then(Value::as_str)
                .unwrap_or("allow");
        }
    }
    effect == "deny"
}
