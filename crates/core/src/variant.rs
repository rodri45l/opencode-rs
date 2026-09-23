//! Provider variant plugin.
//!
//! Ports the observable behaviour of `packages/core/src/plugin/variant.ts`:
//! after catalog sources are applied, a provider+model pair gets generated GLM
//! variants (`high`, `max`) unless an explicit variant with the same id already
//! exists, in which case the explicit variant is preserved.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A model variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// Variant id.
    pub id: String,
    /// Variant request headers.
    pub headers: Value,
    /// Variant request body.
    pub body: Value,
}

/// The variant plugin.
#[derive(Debug, Default)]
pub struct VariantPlugin;

impl VariantPlugin {
    /// Apply generated variants after existing catalog variants.
    pub fn apply(_existing: Vec<Variant>) -> CoreResult<Vec<Variant>> {
        Err(CoreError::NotImplemented("variant::VariantPlugin::apply"))
    }
}
