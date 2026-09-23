//! Provider variant plugin.
//!
//! Ports the observable behaviour of `packages/core/src/plugin/variant.ts`:
//! after catalog sources are applied, a provider+model pair gets generated GLM
//! variants (`high`, `max`) unless an explicit variant with the same id already
//! exists, in which case the explicit variant is preserved.

use serde_json::{json, Value};

use crate::CoreResult;

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
    pub fn apply(existing: Vec<Variant>) -> CoreResult<Vec<Variant>> {
        let mut variants = existing;
        for (id, body) in [
            ("high", json!({ "reasoning_effort": "high" })),
            ("max", json!({ "reasoning_effort": "max" })),
        ] {
            if variants.iter().any(|variant| variant.id == id) {
                continue;
            }
            variants.push(Variant {
                id: id.to_string(),
                headers: json!({}),
                body,
            });
        }
        Ok(variants)
    }
}
