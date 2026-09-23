//! Port of packages/core/test/model.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a model selection decodes without requiring a variant and
//! preserves an explicit variant.

use opencode_core::model::{decode_model_ref, ModelId, ProviderId, VariantId};
use serde_json::json;

const NOTE: &str = "porting: model not implemented";

#[test]
#[ignore = "porting: model not implemented"]
fn accepts_a_model_selection_without_a_variant() {
    let decoded =
        decode_model_ref(&json!({ "id": "claude-sonnet", "providerID": "anthropic" })).expect(NOTE);

    assert_eq!(decoded.id, ModelId::make("claude-sonnet"));
    assert_eq!(decoded.provider_id, ProviderId::make("anthropic"));
    assert_eq!(decoded.variant, None);
}

#[test]
#[ignore = "porting: model not implemented"]
fn preserves_an_explicit_model_variant() {
    let decoded = decode_model_ref(&json!({
        "id": "claude-sonnet",
        "providerID": "anthropic",
        "variant": "high",
    }))
    .expect(NOTE);

    assert_eq!(decoded.id, ModelId::make("claude-sonnet"));
    assert_eq!(decoded.provider_id, ProviderId::make("anthropic"));
    assert_eq!(decoded.variant, Some(VariantId::make("high")));
}
