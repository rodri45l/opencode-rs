//! Provider option lowering.
//!
//! Ports the observable behaviour of
//! `packages/core/src/v1/config/provider-options.ts`: each provider package maps
//! its configured provider and request options into SDK-shaped `url`, `headers`,
//! `body`, `settings`, and request body fields, falling back to raw lowering for
//! unknown packages.

use serde_json::{Map, Value};

use crate::{CoreError, CoreResult};

/// A provider/request option lowerer for one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lowerer {
    /// The provider package name.
    pub package: String,
}

/// Provider option lowering entry point.
#[derive(Debug, Default)]
pub struct ConfigProviderOptionsV1;

impl ConfigProviderOptionsV1 {
    /// Get the lowerer for a provider package.
    pub fn get(package: &str) -> Lowerer {
        Lowerer {
            package: package.to_string(),
        }
    }
}

impl Lowerer {
    /// Lower configured provider options.
    pub fn provider(&self, options: &Value) -> CoreResult<Value> {
        let options = as_object(options);
        match self.package.as_str() {
            "@ai-sdk/openai" => Ok(openai_provider(&options)),
            "@ai-sdk/anthropic" | "@ai-sdk/google-vertex/anthropic" => {
                Ok(anthropic_provider(&options))
            }
            "@ai-sdk/google" | "@ai-sdk/google-vertex" => Ok(google_provider(&options)),
            "@ai-sdk/azure" => Ok(azure_provider(&options)),
            "@ai-sdk/amazon-bedrock" => Ok(Value::Object(direct(&options, &[]))),
            "@ai-sdk/openai-compatible"
            | "@ai-sdk/cerebras"
            | "@ai-sdk/deepinfra"
            | "@ai-sdk/groq"
            | "@ai-sdk/mistral"
            | "@ai-sdk/togetherai"
            | "@ai-sdk/xai"
            | "@openrouter/ai-sdk-provider"
            | "ai-gateway-provider"
            | "venice-ai-sdk-provider" => Ok(openai_compatible_provider(&options)),
            _ => {
                let mut body = Map::new();
                body.insert("body".to_string(), Value::Object(options));
                Ok(Value::Object(body))
            }
        }
    }

    /// Lower configured request options.
    pub fn request(&self, options: &Value) -> CoreResult<Value> {
        let options = as_object(options);
        match self.package.as_str() {
            "@ai-sdk/openai" | "@ai-sdk/azure" => Ok(openai_request(&options)),
            "@ai-sdk/anthropic" | "@ai-sdk/google-vertex/anthropic" => {
                Ok(anthropic_request(&options))
            }
            "@ai-sdk/google" | "@ai-sdk/google-vertex" => Ok(google_request(&options)),
            "@ai-sdk/amazon-bedrock" => {
                let mut out = Map::new();
                out.insert(
                    "additionalModelRequestFields".to_string(),
                    Value::Object(options),
                );
                Ok(Value::Object(out))
            }
            "@ai-sdk/openai-compatible"
            | "@ai-sdk/cerebras"
            | "@ai-sdk/deepinfra"
            | "@ai-sdk/groq"
            | "@ai-sdk/mistral"
            | "@ai-sdk/togetherai"
            | "@ai-sdk/xai"
            | "@openrouter/ai-sdk-provider"
            | "ai-gateway-provider"
            | "venice-ai-sdk-provider" => Ok(openai_compatible_request(&options)),
            _ => Ok(Value::Object(options)),
        }
    }
}

fn openai_provider(options: &Map<String, Value>) -> Value {
    let mut headers = Map::new();
    if let Some(auth) = bearer(options.get("apiKey")) {
        headers.insert("Authorization".to_string(), Value::String(auth));
    }
    if let Some(organization) = as_string(options.get("organization")) {
        headers.insert(
            "OpenAI-Organization".to_string(),
            Value::String(organization),
        );
    }
    if let Some(project) = as_string(options.get("project")) {
        headers.insert("OpenAI-Project".to_string(), Value::String(project));
    }
    merge_string_headers(&mut headers, options.get("headers"));

    let mut out = Map::new();
    if let Some(url) = as_string(options.get("baseURL")) {
        out.insert("url".to_string(), Value::String(url));
    }
    if !headers.is_empty() {
        out.insert("headers".to_string(), Value::Object(headers));
    }
    if let Some(body) = as_object_entry(options.get("body")) {
        out.insert("body".to_string(), Value::Object(body));
    }
    let settings = omit(
        options,
        &[
            "apiKey",
            "baseURL",
            "organization",
            "project",
            "headers",
            "body",
        ],
    );
    out.insert("settings".to_string(), Value::Object(settings));
    Value::Object(out)
}

fn openai_request(options: &Map<String, Value>) -> Value {
    let mut result = snake(options);
    if options.contains_key("reasoningEffort") || options.contains_key("reasoningSummary") {
        let mut reasoning = match result.get("reasoning") {
            Some(Value::Object(map)) => map.clone(),
            _ => Map::new(),
        };
        if let Some(effort) = options.get("reasoningEffort") {
            reasoning.insert("effort".to_string(), effort.clone());
        }
        if let Some(summary) = options.get("reasoningSummary") {
            reasoning.insert("summary".to_string(), summary.clone());
        }
        result.insert("reasoning".to_string(), Value::Object(reasoning));
        result.remove("reasoning_effort");
        result.remove("reasoning_summary");
    }
    if let Some(verbosity) = options.get("textVerbosity") {
        let mut text = match result.get("text") {
            Some(Value::Object(map)) => map.clone(),
            _ => Map::new(),
        };
        text.insert("verbosity".to_string(), verbosity.clone());
        result.insert("text".to_string(), Value::Object(text));
        result.remove("text_verbosity");
    }
    Value::Object(result)
}

fn anthropic_provider(options: &Map<String, Value>) -> Value {
    let mut headers = Map::new();
    if let Some(api_key) = as_string(options.get("apiKey")) {
        headers.insert("x-api-key".to_string(), Value::String(api_key));
    }
    if let Some(auth) = bearer(options.get("authToken")) {
        headers.insert("Authorization".to_string(), Value::String(auth));
    }
    merge_string_headers(&mut headers, options.get("headers"));

    let mut out = Map::new();
    if let Some(url) = as_string(options.get("baseURL")) {
        out.insert("url".to_string(), Value::String(url));
    }
    if !headers.is_empty() {
        out.insert("headers".to_string(), Value::Object(headers));
    }
    if let Some(body) = as_object_entry(options.get("body")) {
        out.insert("body".to_string(), Value::Object(body));
    }
    let settings = omit(
        options,
        &["apiKey", "authToken", "baseURL", "headers", "body"],
    );
    out.insert("settings".to_string(), Value::Object(settings));
    Value::Object(out)
}

fn anthropic_request(options: &Map<String, Value>) -> Value {
    let mut result = snake(options);
    if options.contains_key("effort") || options.contains_key("taskBudget") {
        let mut output_config = Map::new();
        if let Some(effort) = options.get("effort") {
            output_config.insert("effort".to_string(), effort.clone());
        }
        if let Some(budget) = options.get("taskBudget") {
            output_config.insert("task_budget".to_string(), budget.clone());
        }
        result.insert("output_config".to_string(), Value::Object(output_config));
        result.remove("effort");
        result.remove("task_budget");
    }
    if let Some(Value::Object(metadata)) = options.get("metadata") {
        if let Some(user_id) = metadata.get("userId") {
            let mut merged = match result.get("metadata") {
                Some(Value::Object(map)) => map.clone(),
                _ => Map::new(),
            };
            merged.insert("user_id".to_string(), user_id.clone());
            result.insert("metadata".to_string(), Value::Object(merged));
        }
    }
    Value::Object(result)
}

fn google_provider(options: &Map<String, Value>) -> Value {
    let mut headers = Map::new();
    if let Some(api_key) = as_string(options.get("apiKey")) {
        headers.insert("x-goog-api-key".to_string(), Value::String(api_key));
    }
    merge_string_headers(&mut headers, options.get("headers"));

    let mut out = Map::new();
    if let Some(url) = as_string(options.get("baseURL")) {
        out.insert("url".to_string(), Value::String(url));
    }
    if !headers.is_empty() {
        out.insert("headers".to_string(), Value::Object(headers));
    }
    if let Some(body) = as_object_entry(options.get("body")) {
        out.insert("body".to_string(), Value::Object(body));
    }
    let settings = omit(options, &["apiKey", "baseURL", "headers", "body"]);
    out.insert("settings".to_string(), Value::Object(settings));
    Value::Object(out)
}

fn google_request(options: &Map<String, Value>) -> Value {
    const KEYS: &[&str] = &[
        "thinkingConfig",
        "responseModalities",
        "mediaResolution",
        "imageConfig",
    ];
    let generation = pick(options, KEYS);
    let mut out = omit(options, KEYS);
    if !generation.is_empty() {
        out.insert("generationConfig".to_string(), Value::Object(generation));
    }
    Value::Object(out)
}

fn azure_provider(options: &Map<String, Value>) -> Value {
    let mut headers = Map::new();
    if let Some(api_key) = as_string(options.get("apiKey")) {
        headers.insert("api-key".to_string(), Value::String(api_key));
    }
    merge_string_headers(&mut headers, options.get("headers"));

    let mut out = Map::new();
    if let Some(url) = as_string(options.get("baseURL")) {
        out.insert("url".to_string(), Value::String(url));
    }
    if !headers.is_empty() {
        out.insert("headers".to_string(), Value::Object(headers));
    }
    if let Some(body) = as_object_entry(options.get("body")) {
        out.insert("body".to_string(), Value::Object(body));
    }
    let settings = omit(options, &["apiKey", "baseURL", "headers", "body"]);
    out.insert("settings".to_string(), Value::Object(settings));
    Value::Object(out)
}

fn openai_compatible_provider(options: &Map<String, Value>) -> Value {
    let mut out = direct(options, &["baseURL"]);
    if let Some(url) = as_string(options.get("baseURL")) {
        out.insert("url".to_string(), Value::String(url));
    }
    Value::Object(out)
}

fn openai_compatible_request(options: &Map<String, Value>) -> Value {
    let mut result = options.clone();
    if let Some(effort) = options.get("reasoningEffort") {
        result.insert("reasoning_effort".to_string(), effort.clone());
        result.remove("reasoningEffort");
    }
    Value::Object(result)
}

fn direct(options: &Map<String, Value>, extra_keys: &[&str]) -> Map<String, Value> {
    let mut out = Map::new();
    let mut headers = Map::new();
    merge_string_headers(&mut headers, options.get("headers"));
    if !headers.is_empty() {
        out.insert("headers".to_string(), Value::Object(headers));
    }
    if let Some(body) = as_object_entry(options.get("body")) {
        out.insert("body".to_string(), Value::Object(body));
    }
    let mut omit_keys: Vec<&str> = vec!["headers", "body"];
    omit_keys.extend_from_slice(extra_keys);
    out.insert(
        "settings".to_string(),
        Value::Object(omit(options, &omit_keys)),
    );
    out
}

fn snake(options: &Map<String, Value>) -> Map<String, Value> {
    options
        .iter()
        .map(|(key, value)| (snake_key(key), snake_value(value)))
        .collect()
}

fn snake_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(snake_value).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (snake_key(key), snake_value(value)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn snake_key(key: &str) -> String {
    let mut out = String::new();
    for ch in key.chars() {
        if ch.is_ascii_uppercase() {
            out.push('_');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn omit(options: &Map<String, Value>, keys: &[&str]) -> Map<String, Value> {
    options
        .iter()
        .filter(|(key, _)| !keys.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn pick(options: &Map<String, Value>, keys: &[&str]) -> Map<String, Value> {
    options
        .iter()
        .filter(|(key, _)| keys.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn merge_string_headers(target: &mut Map<String, Value>, value: Option<&Value>) {
    if let Some(Value::Object(headers)) = value {
        for (key, value) in headers {
            if value.is_string() {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn as_object(value: &Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

fn as_object_entry(value: Option<&Value>) -> Option<Map<String, Value>> {
    value.and_then(Value::as_object).cloned()
}

fn as_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn bearer(value: Option<&Value>) -> Option<String> {
    as_string(value).map(|key| format!("Bearer {key}"))
}

/// Error for options that are not an object.
pub fn invalid_options(package: &str) -> CoreError {
    CoreError::Invalid(format!("invalid provider options for {package}"))
}
