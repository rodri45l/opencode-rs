//! Model selection references.
//!
//! Ports the observable shape of `packages/core/src/model.ts` (`ModelV2.Ref`):
//! a model selection is a `(providerID, id)` pair with an optional variant.

use std::fmt;

use crate::{CoreError, CoreResult};

/// Identifier for a model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModelId(String);

impl ModelId {
    /// Construct a model id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier for a provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProviderId(String);

impl ProviderId {
    /// Construct a provider id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier for a model variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VariantId(String);

impl VariantId {
    /// Construct a variant id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VariantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A decoded model selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRef {
    /// Model id.
    pub id: ModelId,
    /// Provider id.
    pub provider_id: ProviderId,
    /// Optional variant.
    pub variant: Option<VariantId>,
}

/// Decode a `ModelV2.Ref` from its wire shape.
pub fn decode_model_ref(_value: &serde_json::Value) -> CoreResult<ModelRef> {
    Err(CoreError::NotImplemented("model::decode_model_ref"))
}
