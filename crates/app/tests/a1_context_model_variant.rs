//! Port of packages/app/src/context/model-variant.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct ModelRef {
    provider_id: String,
    model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Agent {
    model: ModelRef,
    variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Model {
    provider_id: String,
    model_id: String,
    variants: Vec<String>,
}

// `selected` models JS undefined/null/value: None = undefined, Some(None) = null.
fn get_configured_agent_variant(_agent: &Agent, _model: &Model) -> Option<String> {
    None
}

fn resolve_model_variant(
    _variants: &[String],
    _selected: Option<Option<String>>,
    _configured: Option<String>,
) -> Option<String> {
    None
}

fn cycle_model_variant(
    _variants: &[String],
    _selected: Option<Option<String>>,
    _configured: Option<String>,
) -> Option<String> {
    None
}

fn model(provider: &str, id: &str) -> Model {
    Model {
        provider_id: provider.into(),
        model_id: id.into(),
        variants: vec!["low".into(), "high".into(), "xhigh".into()],
    }
}

#[test]
#[ignore = "porting: context/model-variant not implemented"]
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
#[ignore = "porting: context/model-variant not implemented"]
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
#[ignore = "porting: context/model-variant not implemented"]
fn prefers_selected_variant_over_configured_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        resolve_model_variant(&variants, Some(Some("high".into())), Some("xhigh".into())),
        Some("high".to_string())
    );
}

#[test]
#[ignore = "porting: context/model-variant not implemented"]
fn lets_an_explicit_default_override_the_configured_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        resolve_model_variant(&variants, Some(None), Some("xhigh".into())),
        None
    );
}

#[test]
#[ignore = "porting: context/model-variant not implemented"]
fn cycles_from_configured_variant_to_next() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, None, Some("high".into())),
        Some("xhigh".to_string())
    );
}

#[test]
#[ignore = "porting: context/model-variant not implemented"]
fn wraps_from_configured_last_variant_to_first() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, None, Some("xhigh".into())),
        Some("low".to_string())
    );
}

#[test]
#[ignore = "porting: context/model-variant not implemented"]
fn cycles_from_an_explicit_default_to_the_first_variant() {
    let variants = vec!["low".into(), "high".into(), "xhigh".into()];
    assert_eq!(
        cycle_model_variant(&variants, Some(None), Some("xhigh".into())),
        Some("low".to_string())
    );
}
