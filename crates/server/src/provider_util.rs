//! Provider utilities: model parsing, sorting, reasoning variants, and the
//! public-info projection.
//!
//! Ports the observable behaviour of the pure synchronous helpers in
//! `packages/opencode/src/provider/provider.ts` and
//! `packages/opencode/src/provider/transform.ts`.

use serde_json::{json, Map, Value};

/// A typed provider error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Parse a `provider/model` string, keeping any slashes in the model id.
pub fn parse_model(input: &str) -> Result<Value, ProviderError> {
    let (provider, model) = match input.split_once('/') {
        Some((provider, model)) => (provider, model),
        None => (input, ""),
    };
    Ok(json!({ "providerID": provider, "modelID": model }))
}

const PRIORITY: [&str; 4] = ["gpt-5", "claude-sonnet-4", "big-pickle", "gemini-3-pro"];

/// Sort models by preferred family, `latest` marker, then id.
pub fn provider_sort(models: &[Value]) -> Result<Vec<Value>, ProviderError> {
    let mut sorted = models.to_vec();
    sorted.sort_by(|a, b| {
        let a_id = a.get("id").and_then(Value::as_str).unwrap_or_default();
        let b_id = b.get("id").and_then(Value::as_str).unwrap_or_default();
        let priority = |id: &str| {
            PRIORITY
                .iter()
                .position(|filter| id.contains(filter))
                .map(|index| index as i64)
                .unwrap_or(-1)
        };
        let latest = |id: &str| if id.contains("latest") { 0 } else { 1 };
        priority(b_id)
            .cmp(&priority(a_id))
            .then_with(|| latest(a_id).cmp(&latest(b_id)))
            .then_with(|| b_id.cmp(a_id))
    });
    Ok(sorted)
}

/// Derive reasoning variants from models.dev `reasoning_options`.
pub fn reasoning_variants(reasoning_options: &Value, npm: &str) -> Result<Value, ProviderError> {
    let options = reasoning_options.as_array().cloned().unwrap_or_default();
    if options.is_empty() {
        return Ok(json!({}));
    }

    if let Some(effort) = options
        .iter()
        .find(|option| option.get("type").and_then(Value::as_str) == Some("effort"))
    {
        let values = effort
            .get("values")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut variants = Map::new();
        for value in values {
            let id = if value.is_null() {
                Some("none".to_string())
            } else {
                value.as_str().map(str::to_string)
            };
            let Some(id) = id else {
                continue;
            };
            if let Some(settings) = effort_settings(npm, &id) {
                variants.insert(id, settings);
            }
        }
        return Ok(Value::Object(variants));
    }

    let toggle = options
        .iter()
        .any(|option| option.get("type").and_then(Value::as_str) == Some("toggle"));
    if toggle && npm == "@ai-sdk/openai" {
        let mut variants = Map::new();
        for effort in ["none", "low", "medium", "high", "xhigh"] {
            variants.insert(
                effort.to_string(),
                effort_settings(npm, effort).unwrap_or(Value::Null),
            );
        }
        return Ok(Value::Object(variants));
    }

    Ok(json!({}))
}

fn effort_settings(npm: &str, effort: &str) -> Option<Value> {
    match npm {
        "@ai-sdk/openai" | "@ai-sdk/azure" | "@ai-sdk/amazon-bedrock/mantle" => Some(json!({
            "reasoningEffort": effort,
            "reasoningSummary": "auto",
            "include": ["reasoning.encrypted_content"],
        })),
        "@ai-sdk/google" | "@ai-sdk/google-vertex" => Some(json!({
            "thinkingConfig": { "includeThoughts": true, "thinkingLevel": effort },
        })),
        "@ai-sdk/anthropic" | "@ai-sdk/google-vertex/anthropic" => {
            Some(json!({ "effort": effort }))
        }
        "merge-gateway-ai-sdk-provider" | "ai-gateway-provider" => {
            Some(json!({ "reasoningEffort": effort }))
        }
        _ => Some(json!({ "reasoningEffort": effort })),
    }
}

/// Drop models whose declared cost is not a finite number.
pub fn to_public_info(models: &Value) -> Result<Value, ProviderError> {
    let Some(object) = models.as_object() else {
        return Ok(json!({}));
    };
    let mut result = Map::new();
    for (key, model) in object {
        let valid = model
            .get("cost")
            .and_then(|cost| cost.get("input"))
            .and_then(Value::as_f64)
            .is_some()
            && model
                .get("cost")
                .and_then(|cost| cost.get("output"))
                .and_then(Value::as_f64)
                .is_some();
        if valid {
            result.insert(key.clone(), model.clone());
        }
    }
    Ok(Value::Object(result))
}

fn sdk_key(npm: &str) -> Option<&'static str> {
    match npm {
        "@ai-sdk/github-copilot" => Some("copilot"),
        "@ai-sdk/azure" => Some("azure"),
        "@ai-sdk/openai" | "@ai-sdk/amazon-bedrock/mantle" => Some("openai"),
        "@ai-sdk/amazon-bedrock" => Some("bedrock"),
        "@ai-sdk/anthropic" | "@ai-sdk/google-vertex/anthropic" => Some("anthropic"),
        "@ai-sdk/google-vertex" => Some("vertex"),
        "@ai-sdk/google" => Some("google"),
        "@ai-sdk/alibaba" => Some("alibaba"),
        "@ai-sdk/cerebras" => Some("cerebras"),
        "@ai-sdk/cohere" => Some("cohere"),
        "@ai-sdk/deepinfra" => Some("deepinfra"),
        "@ai-sdk/groq" => Some("groq"),
        "@ai-sdk/mistral" => Some("mistral"),
        "@ai-sdk/perplexity" => Some("perplexity"),
        "@ai-sdk/togetherai" => Some("togetherai"),
        "@ai-sdk/vercel" => Some("vercel"),
        "@ai-sdk/xai" => Some("xai"),
        "venice-ai-sdk-provider" => Some("venice"),
        "@ai-sdk/gateway" => Some("gateway"),
        "@openrouter/ai-sdk-provider" => Some("openrouter"),
        "merge-gateway-ai-sdk-provider" => Some("mergeGateway"),
        "ai-gateway-provider" => Some("openaiCompatible"),
        _ => None,
    }
}

/// Wrap `options` under the AI SDK provider key for `model`.
pub fn provider_options(model: &Value, options: &Value) -> Result<Value, ProviderError> {
    let npm = model
        .get("api")
        .and_then(|api| api.get("npm"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(key) = sdk_key(npm) else {
        return Ok(json!({}));
    };
    let mut map = Map::new();
    map.insert(key.to_string(), options.clone());
    Ok(Value::Object(map))
}

const OPENAI_EFFORTS: [&str; 6] = ["none", "minimal", "low", "medium", "high", "xhigh"];

/// The generated reasoning variants for `model`.
pub fn transform_variants(model: &Value) -> Result<Value, ProviderError> {
    let npm = model
        .get("api")
        .and_then(|api| api.get("npm"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(npm, "@ai-sdk/openai" | "@ai-sdk/amazon-bedrock/mantle") {
        return Ok(json!({}));
    }
    let mut variants = Map::new();
    for effort in OPENAI_EFFORTS {
        if let Some(settings) = effort_settings(npm, effort) {
            variants.insert(effort.to_string(), settings);
        }
    }
    Ok(Value::Object(variants))
}

/// Derive reasoning variants for `model` from models.dev options.
pub fn transform_reasoning_variants(
    reasoning_options: &Value,
    model: &Value,
) -> Result<Option<Value>, ProviderError> {
    let npm = model
        .get("api")
        .and_then(|api| api.get("npm"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let variants = reasoning_variants(reasoning_options, npm)?;
    if variants
        .as_object()
        .map(|object| object.is_empty())
        .unwrap_or(true)
    {
        Ok(None)
    } else {
        Ok(Some(variants))
    }
}
