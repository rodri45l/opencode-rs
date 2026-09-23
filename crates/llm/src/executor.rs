//! Request executor classification, retry policy and redaction.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, ProviderHttpError};
use crate::provider_error::is_context_overflow;
use crate::route::ExecutedResponse;

const MAX_BODY: usize = 16_384;

fn secret_keys() -> [&'static str; 7] {
    [
        "authorization",
        "api-key",
        "x-api-key",
        "apikey",
        "token",
        "secret",
        "password",
    ]
}

fn looks_secret(name: &str) -> bool {
    let lower = name.to_lowercase();
    secret_keys().iter().any(|key| lower.contains(key)) || lower.contains("key")
}

fn collect_secrets(request: &Value) -> Vec<String> {
    let mut secrets = Vec::new();
    if let Some(url) = request.get("url").and_then(Value::as_str) {
        if let Some((_, query)) = url.split_once('?') {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    if looks_secret(key) && !value.is_empty() {
                        secrets.push(value.to_string());
                    }
                }
            }
        }
    }
    if let Some(headers) = request.get("headers").and_then(Value::as_object) {
        for (name, value) in headers {
            if let Some(value) = value.as_str() {
                if looks_secret(name) {
                    secrets.push(value.to_string());
                    if let Some(rest) = value.strip_prefix("Bearer ") {
                        secrets.push(rest.to_string());
                    }
                }
            }
        }
    }
    secrets
}

fn redact(text: &str, secrets: &[String]) -> String {
    let mut output = text.to_string();
    for secret in secrets {
        if !secret.is_empty() {
            output = output.replace(secret, "<redacted>");
        }
    }
    output
}

fn truncate(body: &str) -> String {
    if body.len() > MAX_BODY {
        let mut end = MAX_BODY;
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}\n[truncated: {MAX_BODY}]", &body[..end])
    } else {
        body.to_string()
    }
}

/// Execute one request/response pair with classification.
pub fn execute(input: Value) -> Result<ExecutedResponse, LlmError> {
    let request = input.get("request").cloned().unwrap_or_else(|| json!({}));
    let response = input.get("response").cloned().unwrap_or_else(|| json!({}));
    let status = response.get("status").and_then(Value::as_u64).unwrap_or(0) as u16;
    let body = response
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if (200..300).contains(&status) {
        if response
            .get("streamParseError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(LlmError::InvalidRequest(
                "InvalidProviderOutput: failed to parse provider stream".into(),
            ));
        }
        return Ok(ExecutedResponse { status, body });
    }

    let secrets = collect_secrets(&request);
    let redacted_body = redact(&truncate(&body), &secrets);
    let classification = if is_context_overflow(&body) {
        Some("context-overflow")
    } else {
        None
    };
    let retryable = matches!(status, 429 | 504 | 529) || status >= 500;

    let mut diagnostics = Map::new();
    diagnostics.insert("status".into(), json!(status));
    if let Some(method) = request.get("method") {
        diagnostics.insert("method".into(), method.clone());
    }
    if let Some(url) = request.get("url").and_then(Value::as_str) {
        diagnostics.insert("url".into(), Value::String(redact(url, &secrets)));
    }
    if let Some(headers) = response.get("headers").and_then(Value::as_object) {
        let mut rendered = Map::new();
        for (name, value) in headers {
            let value = value
                .as_str()
                .map(|v| redact(v, &secrets))
                .unwrap_or_default();
            rendered.insert(name.clone(), Value::String(value));
        }
        diagnostics.insert("headers".into(), Value::Object(rendered));
    }
    diagnostics.insert("body".into(), Value::String(redacted_body.clone()));

    let message = format!("HTTP {status}: {redacted_body}");
    Err(LlmError::ProviderHttp(Box::new(ProviderHttpError {
        message,
        classification,
        retryable,
        diagnostics: serde_json::to_string(&Value::Object(diagnostics)).unwrap_or_default(),
    })))
}
