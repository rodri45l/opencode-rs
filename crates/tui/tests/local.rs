//! Port of packages/tui/test/context/local.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/context/local.tsx; see docs/TEST-PORT.md.

use opencode_tui::local::{parse_model, recent_models, ModelRef};

fn model(index: usize) -> ModelRef {
    ModelRef {
        provider_id: "provider".to_string(),
        model_id: format!("model-{index}"),
    }
}

#[test]
fn parses_model_ids_containing_slashes() {
    assert_eq!(
        parse_model("provider/family/model"),
        ModelRef {
            provider_id: "provider".to_string(),
            model_id: "family/model".to_string(),
        }
    );
}

#[test]
fn moves_a_model_to_the_front_deduplicates_and_limits_recents() {
    let recent: Vec<ModelRef> = (0..12).map(model).collect();
    let expected: Vec<ModelRef> = std::iter::once(model(5))
        .chain((0..5).map(model))
        .chain((6..10).map(model))
        .collect();

    assert_eq!(recent_models(model(5), &recent), expected);
}
