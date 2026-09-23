//! Port of packages/opencode/test/acp/config-option.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/config-option.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure option builders (`buildModelSelectOption`,
//! `buildEffortSelectOption`, `buildModeSelectOption`, `buildConfigOptions`) and
//! `parseModelSelection` / `formatCurrentModelId` / `formatVariantName`.
//! Dropped: none — every upstream case in this file is pure.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn providers() -> Value {
    json!([
        {
            "id": "anthropic",
            "name": "Anthropic",
            "models": {
                "claude/sonnet-4": {
                    "id": "claude/sonnet-4",
                    "name": "Claude Sonnet 4",
                    "variants": { "default": {}, "high": {}, "very-high": {} }
                },
                "claude-haiku": { "id": "claude-haiku", "name": "Claude Haiku" }
            }
        },
        {
            "id": "openai",
            "name": "OpenAI",
            "models": {
                "gpt-5": {
                    "id": "gpt-5",
                    "name": "GPT-5",
                    "variants": { "minimal": {}, "low": {} }
                }
            }
        }
    ])
}

fn model(provider_id: &str, model_id: &str) -> Value {
    json!({ "providerID": provider_id, "modelID": model_id })
}

fn build_model_select_option(
    _providers: &Value,
    _current_model: &Value,
    _current_variant: Option<&str>,
    _include_variants: bool,
) -> Result<Value, NotImplemented> {
    nope("acp config-option")
}

fn build_effort_select_option(
    _variants: &[&str],
    _current_variant: Option<&str>,
) -> Result<Option<Value>, NotImplemented> {
    nope("acp config-option")
}

fn build_mode_select_option(
    _current_mode_id: &str,
    _modes: &Value,
) -> Result<Value, NotImplemented> {
    nope("acp config-option")
}

fn build_config_options(
    _providers: &Value,
    _current_model: &Value,
    _current_variant: Option<&str>,
    _modes: &Value,
    _current_mode_id: Option<&str>,
) -> Result<Vec<Value>, NotImplemented> {
    nope("acp config-option")
}

fn parse_model_selection(_selection: &str, _providers: &Value) -> Result<Value, NotImplemented> {
    nope("acp config-option")
}

fn format_current_model_id(
    _model: &Value,
    _variant: Option<&str>,
    _variants: &[&str],
    _include_variant: bool,
) -> Result<String, NotImplemented> {
    nope("acp config-option")
}

fn format_variant_name(_variant: &str) -> Result<String, NotImplemented> {
    nope("acp config-option")
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn builds_the_model_select_option_with_acp_verifier_category() {
    let option = build_model_select_option(
        &providers(),
        &model("anthropic", "claude/sonnet-4"),
        Some("high"),
        false,
    )
    .unwrap();
    assert_eq!(
        option,
        json!({
            "id": "model",
            "name": "Model",
            "category": "model",
            "type": "select",
            "currentValue": "anthropic/claude/sonnet-4",
            "options": [
                { "value": "anthropic/claude-haiku", "name": "Anthropic/Claude Haiku" },
                { "value": "anthropic/claude/sonnet-4", "name": "Anthropic/Claude Sonnet 4" },
                { "value": "openai/gpt-5", "name": "OpenAI/GPT-5" }
            ]
        })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn includes_variant_ids_in_the_model_option_only_when_requested() {
    let option = build_model_select_option(
        &providers(),
        &model("anthropic", "claude/sonnet-4"),
        Some("high"),
        true,
    )
    .unwrap();
    assert_eq!(
        option["currentValue"],
        json!("anthropic/claude/sonnet-4/high")
    );
    let options = option["options"].as_array().unwrap();
    assert!(options.contains(&json!({
        "value": "anthropic/claude/sonnet-4/high",
        "name": "Anthropic/Claude Sonnet 4 (High)"
    })));
    assert!(!options.contains(&json!({
        "value": "anthropic/claude/sonnet-4/default",
        "name": "Anthropic/Claude Sonnet 4 (Default)"
    })));
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn builds_effort_option_and_falls_back_to_default_when_current_variant_invalid() {
    let option = build_effort_select_option(&["low", "default", "high"], Some("missing")).unwrap();
    assert_eq!(
        option,
        Some(json!({
            "id": "effort",
            "name": "Effort",
            "description": "Available effort levels for this model",
            "category": "thought_level",
            "type": "select",
            "currentValue": "default",
            "options": [
                { "value": "low", "name": "Low" },
                { "value": "default", "name": "Default" },
                { "value": "high", "name": "High" }
            ]
        }))
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn effort_fallback_uses_the_first_variant_when_default_is_absent() {
    let option = build_effort_select_option(&["minimal", "low"], Some("missing")).unwrap();
    assert_eq!(option.unwrap()["currentValue"], json!("minimal"));
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn omits_effort_option_when_there_are_no_variants() {
    assert_eq!(build_effort_select_option(&[], None).unwrap(), None);
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn exposes_an_explicit_default_even_when_the_provider_only_lists_named_variants() {
    let option = build_effort_select_option(&["low", "medium"], Some("default"))
        .unwrap()
        .unwrap();
    assert_eq!(option["currentValue"], json!("default"));
    assert_eq!(
        option["options"],
        json!([
            { "value": "low", "name": "Low" },
            { "value": "medium", "name": "Medium" },
            { "value": "default", "name": "Default" }
        ])
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn builds_the_mode_select_option_with_descriptions_when_present() {
    let option = build_mode_select_option(
        "build",
        &json!([
            { "id": "build", "name": "Build", "description": "Make code changes" },
            { "id": "plan", "name": "Plan" }
        ]),
    )
    .unwrap();
    assert_eq!(
        option,
        json!({
            "id": "mode",
            "name": "Session Mode",
            "category": "mode",
            "type": "select",
            "currentValue": "build",
            "options": [
                { "value": "build", "name": "Build", "description": "Make code changes" },
                { "value": "plan", "name": "Plan" }
            ]
        })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn builds_full_config_options_in_stable_order() {
    let options = build_config_options(
        &providers(),
        &model("anthropic", "claude/sonnet-4"),
        Some("very-high"),
        &json!([{ "id": "build", "name": "Build" }, { "id": "plan", "name": "Plan" }]),
        Some("plan"),
    )
    .unwrap();
    let ids: Vec<_> = options.iter().map(|o| o["id"].clone()).collect();
    let categories: Vec<_> = options.iter().map(|o| o["category"].clone()).collect();
    assert_eq!(ids, vec![json!("model"), json!("effort"), json!("mode")]);
    assert_eq!(
        categories,
        vec![json!("model"), json!("thought_level"), json!("mode")]
    );
    assert_eq!(options[1]["currentValue"], json!("very-high"));
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn full_config_options_omit_effort_for_models_without_variants() {
    let options = build_config_options(
        &providers(),
        &model("anthropic", "claude-haiku"),
        None,
        &json!([]),
        None,
    )
    .unwrap();
    let ids: Vec<_> = options.iter().map(|o| o["id"].clone()).collect();
    assert_eq!(ids, vec![json!("model")]);
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn parses_provider_model_selections() {
    assert_eq!(
        parse_model_selection("openai/gpt-5", &providers()).unwrap(),
        json!({ "model": { "providerID": "openai", "modelID": "gpt-5" } })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn parses_provider_model_variant_selections_when_the_base_model_exposes_that_variant() {
    assert_eq!(
        parse_model_selection("openai/gpt-5/low", &providers()).unwrap(),
        json!({ "model": { "providerID": "openai", "modelID": "gpt-5" }, "variant": "low" })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn prefers_exact_slash_containing_model_ids_before_treating_the_tail_as_a_variant() {
    assert_eq!(
        parse_model_selection("anthropic/claude/sonnet-4", &providers()).unwrap(),
        json!({ "model": { "providerID": "anthropic", "modelID": "claude/sonnet-4" } })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn parses_trailing_variants_for_slash_containing_model_ids() {
    assert_eq!(
        parse_model_selection("anthropic/claude/sonnet-4/high", &providers()).unwrap(),
        json!({ "model": { "providerID": "anthropic", "modelID": "claude/sonnet-4" }, "variant": "high" })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn keeps_unknown_trailing_segments_in_the_model_id_when_they_are_not_valid_variants() {
    assert_eq!(
        parse_model_selection("anthropic/claude/sonnet-4/missing", &providers()).unwrap(),
        json!({ "model": { "providerID": "anthropic", "modelID": "claude/sonnet-4/missing" } })
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn formats_current_model_ids_with_and_without_selected_variants() {
    assert_eq!(
        format_current_model_id(
            &model("openai", "gpt-5"),
            Some("low"),
            &["minimal", "low"],
            false
        )
        .unwrap(),
        "openai/gpt-5"
    );
    assert_eq!(
        format_current_model_id(
            &model("openai", "gpt-5"),
            Some("low"),
            &["minimal", "low"],
            true
        )
        .unwrap(),
        "openai/gpt-5/low"
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn formats_current_model_ids_with_variant_fallback() {
    assert_eq!(
        format_current_model_id(
            &model("anthropic", "claude/sonnet-4"),
            Some("missing"),
            &["default", "high"],
            true
        )
        .unwrap(),
        "anthropic/claude/sonnet-4/default"
    );
}

#[test]
#[ignore = "porting: acp config-option not implemented"]
fn formats_variant_names_for_display() {
    assert_eq!(
        format_variant_name("very_high-effort").unwrap(),
        "Very High Effort"
    );
}
