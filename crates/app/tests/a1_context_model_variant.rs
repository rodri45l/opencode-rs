//! Port of packages/app/src/context/model-variant.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::model_variant::{
    cycle_model_variant, get_configured_agent_variant, resolve_model_variant, Agent, Model,
    ModelRef,
};

fn model(provider: &str, id: &str) -> Model {
    Model {
        provider_id: provider.into(),
        model_id: id.into(),
        variants: vec!["low".into(), "high".into(), "xhigh".into()],
    }
}

#[test]
fn resolves_configured_agent_variant_when_model_matches() {
    let agent = Agent {
        model: ModelRef {
            provider_id: "openai".into(),
            model_id: "gpt-5.2".into(),
        },
        variant: Some("xhigh".into()),
    };
    assert_eq!(
        get_configured_agent_variant(&agent, &model("openai", "gpt-5.2")),
        Some("xhigh".to_string())
    );
}

#[test]
fn ignores_configured_variant_when_model_does_not_match() {
    let agent = Agent {
        model: ModelRef {
            provider_id: "openai".into(),
            model_id: "gpt-5.2".into(),
        },
        variant: Some("xhigh".into()),
    };
    assert_eq!(
        get_configured_agent_variant(&agent, &model("anthropic", "claude-sonnet-4")),
        None
    );
}

#[test]
fn prefers_selected_variant_over_configured_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        resolve_model_variant(&variants, Some(Some("high".into())), Some("xhigh".into())),
        Some("high".to_string())
    );
}

#[test]
fn lets_an_explicit_default_override_the_configured_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        resolve_model_variant(&variants, Some(None), Some("xhigh".into())),
        None
    );
}

#[test]
fn cycles_from_configured_variant_to_next() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, None, Some("high".into())),
        Some("xhigh".to_string())
    );
}

#[test]
fn wraps_from_configured_last_variant_to_first() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, None, Some("xhigh".into())),
        Some("low".to_string())
    );
}

#[test]
fn cycles_from_an_explicit_default_to_the_first_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, Some(None), Some("xhigh".into())),
        Some("low".to_string())
    );
}
