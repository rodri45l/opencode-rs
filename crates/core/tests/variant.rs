//! Port of packages/core/test/plugin/variant.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the GLM 5.2 variant plugin adds `high` and `max` variants
//! after the catalog sources, and an explicit variant with the same id keeps its
//! own headers while the generated `max` is still added. Re-derived: the
//! `Catalog`/`Location`/host wiring is replaced by a direct variant merge.

use opencode_core::variant::{Variant, VariantPlugin};
use serde_json::json;

const NOTE: &str = "porting: variant plugin not implemented";

fn ids(variants: &[Variant]) -> Vec<&str> {
    variants.iter().map(|variant| variant.id.as_str()).collect()
}

#[test]
fn adds_glm_52_variants_after_catalog_sources() {
    let variants = VariantPlugin::apply(vec![]).expect(NOTE);
    assert_eq!(ids(&variants), vec!["high", "max"]);
    assert_eq!(variants[0].body, json!({ "reasoning_effort": "high" }));
    assert_eq!(variants[1].body, json!({ "reasoning_effort": "max" }));
}

#[test]
fn keeps_explicit_variants_over_generated_defaults() {
    let variants = VariantPlugin::apply(vec![Variant {
        id: "high".into(),
        headers: json!({ "custom": "true" }),
        body: json!({}),
    }])
    .expect(NOTE);

    assert_eq!(ids(&variants), vec!["high", "max"]);
    assert_eq!(variants[0].headers, json!({ "custom": "true" }));
    assert_eq!(variants[1].body, json!({ "reasoning_effort": "max" }));
}
