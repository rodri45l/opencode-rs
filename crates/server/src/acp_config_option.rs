//! ACP session config-option builders.
//!
//! Ports the observable behaviour of
//! `packages/opencode/src/acp/config-option.ts`: model/effort/mode select
//! options, model-selection parsing, and current-model/variant formatting.

use serde_json::{json, Value};

/// The default variant sentinel.
pub const DEFAULT_VARIANT_VALUE: &str = "default";

/// A typed ACP config-option error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpConfigError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for AcpConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for AcpConfigError {}

fn providers_of(providers: &Value) -> Vec<Value> {
    providers.as_array().cloned().unwrap_or_default()
}

fn models_of(provider: &Value) -> Vec<Value> {
    provider
        .get("models")
        .and_then(Value::as_object)
        .map(|models| models.values().cloned().collect())
        .unwrap_or_default()
}

fn variants_of(model: &Value) -> Vec<String> {
    model
        .get("variants")
        .and_then(Value::as_object)
        .map(|variants| variants.keys().cloned().collect())
        .unwrap_or_default()
}

fn variants_for_model(providers: &Value, model: &Value) -> Vec<String> {
    let provider_id = model
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let model_id = model
        .get("modelID")
        .and_then(Value::as_str)
        .unwrap_or_default();
    for provider in providers_of(providers) {
        if provider.get("id").and_then(Value::as_str) == Some(provider_id) {
            if let Some(found) = provider
                .get("models")
                .and_then(Value::as_object)
                .and_then(|models| models.get(model_id))
            {
                return variants_of(found);
            }
        }
    }
    Vec::new()
}

/// Format a variant key for display.
pub fn format_variant_name(variant: &str) -> Result<String, AcpConfigError> {
    Ok(format_variant_name_impl(variant))
}

fn format_variant_name_impl(variant: &str) -> String {
    variant
        .split(['_', '-'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn select_variant(variant: Option<&str>, variants: &[String]) -> String {
    if let Some(variant) = variant {
        if variants.iter().any(|item| item == variant) {
            return variant.to_string();
        }
    }
    if variants.iter().any(|item| item == DEFAULT_VARIANT_VALUE) {
        return DEFAULT_VARIANT_VALUE.to_string();
    }
    variants.first().cloned().unwrap_or_default()
}

/// Format the current `provider/model[/variant]` id.
pub fn format_current_model_id(
    model: &Value,
    variant: Option<&str>,
    variants: &[&str],
    include_variant: bool,
) -> Result<String, AcpConfigError> {
    Ok(format_current_model_id_impl(
        model,
        variant,
        variants,
        include_variant,
    ))
}

fn format_current_model_id_impl(
    model: &Value,
    variant: Option<&str>,
    variants: &[&str],
    include_variant: bool,
) -> String {
    let provider_id = model
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let model_id = model
        .get("modelID")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let base = format!("{provider_id}/{model_id}");
    if !include_variant || variants.is_empty() {
        return base;
    }
    let owned: Vec<String> = variants.iter().map(|item| (*item).to_string()).collect();
    format!("{base}/{}", select_variant(variant, &owned))
}

fn build_model_select_options(providers: &Value, include_variants: bool) -> Vec<Value> {
    let mut options = Vec::new();
    for provider in providers_of(providers) {
        let provider_id = provider
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let provider_name = provider
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let mut models = models_of(&provider);
        models.sort_by(|a, b| {
            a.get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .cmp(b.get("name").and_then(Value::as_str).unwrap_or_default())
        });
        for model in models {
            let model_id = model.get("id").and_then(Value::as_str).unwrap_or_default();
            let model_name = model
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            options.push(json!({
                "value": format!("{provider_id}/{model_id}"),
                "name": format!("{provider_name}/{model_name}"),
            }));
            if !include_variants {
                continue;
            }
            for variant in variants_of(&model) {
                if variant == DEFAULT_VARIANT_VALUE {
                    continue;
                }
                options.push(json!({
                    "value": format!("{provider_id}/{model_id}/{variant}"),
                    "name": format!("{provider_name}/{model_name} ({})", format_variant_name_impl(&variant)),
                }));
            }
        }
    }
    options
}

/// Build the `model` select option.
pub fn build_model_select_option(
    providers: &Value,
    current_model: &Value,
    current_variant: Option<&str>,
    include_variants: bool,
) -> Result<Value, AcpConfigError> {
    let variants = variants_for_model(providers, current_model);
    let variant_refs: Vec<&str> = variants.iter().map(String::as_str).collect();
    Ok(json!({
        "id": "model",
        "name": "Model",
        "category": "model",
        "type": "select",
        "currentValue": format_current_model_id_impl(current_model, current_variant, &variant_refs, include_variants),
        "options": build_model_select_options(providers, include_variants),
    }))
}

/// Build the `effort` select option, or `None` when there are no variants.
pub fn build_effort_select_option(
    variants: &[&str],
    current_variant: Option<&str>,
) -> Result<Option<Value>, AcpConfigError> {
    if variants.is_empty() {
        return Ok(None);
    }
    let owned: Vec<String> = variants.iter().map(|item| (*item).to_string()).collect();
    let current_value = if current_variant == Some(DEFAULT_VARIANT_VALUE) {
        DEFAULT_VARIANT_VALUE.to_string()
    } else {
        select_variant(current_variant, &owned)
    };
    let mut all: Vec<String> = owned.clone();
    if !all.iter().any(|item| item == DEFAULT_VARIANT_VALUE) {
        all.push(DEFAULT_VARIANT_VALUE.to_string());
    }
    let options: Vec<Value> = all
        .iter()
        .map(|variant| json!({ "value": variant, "name": format_variant_name_impl(variant) }))
        .collect();
    Ok(Some(json!({
        "id": "effort",
        "name": "Effort",
        "description": "Available effort levels for this model",
        "category": "thought_level",
        "type": "select",
        "currentValue": current_value,
        "options": options,
    })))
}

/// Build the `mode` select option.
pub fn build_mode_select_option(
    current_mode_id: &str,
    modes: &Value,
) -> Result<Value, AcpConfigError> {
    let options: Vec<Value> = modes
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|mode| {
            let mut option = json!({
                "value": mode.get("id").and_then(Value::as_str).unwrap_or_default(),
                "name": mode.get("name").and_then(Value::as_str).unwrap_or_default(),
            });
            if let Some(description) = mode.get("description").and_then(Value::as_str) {
                option["description"] = json!(description);
            }
            option
        })
        .collect();
    Ok(json!({
        "id": "mode",
        "name": "Session Mode",
        "category": "mode",
        "type": "select",
        "currentValue": current_mode_id,
        "options": options,
    }))
}

/// Build the ordered list of config options.
pub fn build_config_options(
    providers: &Value,
    current_model: &Value,
    current_variant: Option<&str>,
    modes: &Value,
    current_mode_id: Option<&str>,
) -> Result<Vec<Value>, AcpConfigError> {
    let variants = variants_for_model(providers, current_model);
    let variant_refs: Vec<&str> = variants.iter().map(String::as_str).collect();
    let effort = build_effort_select_option(&variant_refs, current_variant)?;

    let mut options = vec![build_model_select_option(
        providers,
        current_model,
        current_variant,
        false,
    )?];
    if let Some(effort) = effort {
        options.push(effort);
    }
    if let Some(mode_id) = current_mode_id {
        options.push(build_mode_select_option(mode_id, modes)?);
    }
    Ok(options)
}

/// Parse a `provider/model[/variant]` selection.
pub fn parse_model_selection(model_id: &str, providers: &Value) -> Result<Value, AcpConfigError> {
    for provider in providers_of(providers) {
        let provider_id = provider
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let prefix = format!("{provider_id}/");
        if !model_id.starts_with(&prefix) {
            continue;
        }
        let rest = &model_id[prefix.len()..];
        if provider
            .get("models")
            .and_then(Value::as_object)
            .map(|models| models.contains_key(rest))
            .unwrap_or(false)
        {
            return Ok(json!({ "model": { "providerID": provider_id, "modelID": rest } }));
        }
        if let Some(separator) = rest.rfind('/') {
            let base = &rest[..separator];
            let variant = &rest[separator + 1..];
            let exposes = provider
                .get("models")
                .and_then(Value::as_object)
                .and_then(|models| models.get(base))
                .and_then(|model| model.get("variants"))
                .and_then(Value::as_object)
                .map(|variants| variants.contains_key(variant))
                .unwrap_or(false);
            if exposes {
                return Ok(json!({
                    "model": { "providerID": provider_id, "modelID": base },
                    "variant": variant,
                }));
            }
        }
        return Ok(json!({ "model": { "providerID": provider_id, "modelID": rest } }));
    }

    match model_id.find('/') {
        None => Ok(json!({ "model": { "providerID": model_id, "modelID": "" } })),
        Some(separator) => Ok(json!({
            "model": {
                "providerID": &model_id[..separator],
                "modelID": &model_id[separator + 1..],
            },
        })),
    }
}
