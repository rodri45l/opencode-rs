//! GitHub Copilot Responses conversions (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/github-copilot/responses/convert-to-openai-responses-input.ts`
//! and the `copilot` provider-metadata namespace of
//! `.../openai-responses-language-model.ts`: a tool call echoes a stale
//! `copilot.itemId` as the `function_call` id (omitted once stripped), a
//! reasoning part is preserved only when it carries a `copilot.itemId` (and
//! otherwise dropped with a warning), a user file part reads `imageDetail` from
//! the `copilot` namespace, and generated item/response metadata is attached
//! under the `copilot` namespace rather than `openai`. The `LanguageModelV3`
//! network/runtime and the mock fetch are dropped.

use serde_json::{json, Map, Value};

use crate::{CoreError, CoreResult};

/// The provider-metadata namespace used by the Copilot Responses model.
pub const METADATA_NAMESPACE: &str = "copilot";

/// A converted Responses input plus any warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertedInput {
    /// The OpenAI Responses input items.
    pub input: Vec<Value>,
    /// Conversion warnings.
    pub warnings: Vec<Value>,
}

/// The warning emitted when a reasoning part has no Copilot item id.
pub const REASONING_WARNING: &str = "Non-OpenAI reasoning parts are not supported";

/// Convert a `LanguageModelV3` prompt array into Responses input items.
pub fn convert_to_openai_responses_input(prompt: &Value) -> CoreResult<ConvertedInput> {
    let messages = prompt
        .as_array()
        .ok_or_else(|| CoreError::Invalid("prompt must be an array".into()))?;
    let mut input = Vec::new();
    let mut warnings = Vec::new();
    for message in messages {
        match message.get("role").and_then(Value::as_str) {
            Some("user") => convert_user(message, &mut input),
            Some("assistant") => convert_assistant(message, &mut input, &mut warnings),
            Some("tool") => convert_tool(message, &mut input),
            _ => {}
        }
    }
    Ok(ConvertedInput { input, warnings })
}

fn convert_user(message: &Value, input: &mut Vec<Value>) {
    let mut parts = Vec::new();
    if let Some(Value::Array(content)) = message.get("content") {
        for part in content {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => {
                    parts.push(json!({
                        "type": "input_text",
                        "text": part.get("text").cloned().unwrap_or(Value::Null),
                    }));
                }
                Some("file") => {
                    let media_type = part
                        .get("mediaType")
                        .and_then(Value::as_str)
                        .unwrap_or("image/jpeg");
                    let media_type = if media_type == "image/*" {
                        "image/jpeg"
                    } else {
                        media_type
                    };
                    let data = part.get("data").and_then(Value::as_str).unwrap_or("");
                    let image_url = if data.starts_with("http://") || data.starts_with("https://") {
                        data.to_string()
                    } else {
                        format!("data:{media_type};base64,{data}")
                    };
                    let detail = part
                        .get("providerOptions")
                        .and_then(|options| options.get(METADATA_NAMESPACE))
                        .and_then(|copilot| copilot.get("imageDetail"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    parts.push(json!({
                        "type": "input_image",
                        "image_url": image_url,
                        "detail": detail,
                    }));
                }
                _ => {}
            }
        }
    }
    input.push(json!({ "role": "user", "content": parts }));
}

fn convert_assistant(message: &Value, input: &mut Vec<Value>, warnings: &mut Vec<Value>) {
    if let Some(Value::Array(content)) = message.get("content") {
        for part in content {
            match part.get("type").and_then(Value::as_str) {
                Some("tool-call") => {
                    let mut item = Map::new();
                    item.insert("type".into(), json!("function_call"));
                    item.insert(
                        "call_id".into(),
                        part.get("toolCallId").cloned().unwrap_or(Value::Null),
                    );
                    item.insert(
                        "name".into(),
                        part.get("toolName").cloned().unwrap_or(Value::Null),
                    );
                    item.insert(
                        "arguments".into(),
                        Value::String(
                            serde_json::to_string(part.get("input").unwrap_or(&Value::Null))
                                .unwrap_or_default(),
                        ),
                    );
                    if let Some(item_id) = copilot_field(part, "itemId") {
                        item.insert("id".into(), item_id);
                    }
                    input.push(Value::Object(item));
                }
                Some("reasoning") => {
                    let item_id = copilot_field(part, "itemId");
                    match item_id {
                        Some(item_id) => {
                            let encrypted = copilot_field(part, "reasoningEncryptedContent");
                            let text = part.get("text").and_then(Value::as_str).unwrap_or("");
                            let mut summary = Vec::new();
                            if !text.is_empty() {
                                summary.push(json!({ "type": "summary_text", "text": text }));
                            }
                            let mut item = Map::new();
                            item.insert("type".into(), json!("reasoning"));
                            item.insert("id".into(), item_id);
                            item.insert(
                                "encrypted_content".into(),
                                encrypted.unwrap_or(Value::Null),
                            );
                            item.insert("summary".into(), Value::Array(summary));
                            input.push(Value::Object(item));
                        }
                        None => warnings.push(Value::String(format!(
                            "{REASONING_WARNING}. Skipping reasoning part."
                        ))),
                    }
                }
                _ => {}
            }
        }
    }
}

fn convert_tool(message: &Value, input: &mut Vec<Value>) {
    if let Some(Value::Array(content)) = message.get("content") {
        for part in content {
            if part.get("type").and_then(Value::as_str) != Some("tool-result") {
                continue;
            }
            let output = part.get("output");
            let value = output
                .and_then(|output| output.get("value"))
                .cloned()
                .unwrap_or(Value::Null);
            let content_value = match output
                .and_then(|output| output.get("type"))
                .and_then(Value::as_str)
            {
                Some("text") | Some("error-text") => value.as_str().unwrap_or("").to_string(),
                _ => serde_json::to_string(&value).unwrap_or_default(),
            };
            input.push(json!({
                "type": "function_call_output",
                "call_id": part.get("toolCallId").cloned().unwrap_or(Value::Null),
                "output": content_value,
            }));
        }
    }
}

fn copilot_field(part: &Value, field: &str) -> Option<Value> {
    part.get("providerOptions")
        .and_then(|options| options.get(METADATA_NAMESPACE))
        .and_then(|copilot| copilot.get(field))
        .cloned()
}

/// Provider metadata for a generated item, namespaced under `copilot`.
#[derive(Debug, Default)]
pub struct ResponsesMetadata;

impl ResponsesMetadata {
    /// Provider metadata for a reasoning item.
    pub fn reasoning(item_id: &str, encrypted_content: Option<&str>) -> CoreResult<Value> {
        Ok(reasoning_metadata(item_id, encrypted_content))
    }

    /// Provider metadata for a message or function-call item.
    pub fn item(item_id: &str) -> CoreResult<Value> {
        Ok(json!({ METADATA_NAMESPACE: { "itemId": item_id } }))
    }

    /// Response-level provider metadata.
    pub fn response(response_id: &str) -> CoreResult<Value> {
        Ok(json!({ METADATA_NAMESPACE: { "responseId": response_id } }))
    }
}

/// The metadata for a reasoning item carrying an encrypted summary.
pub fn reasoning_metadata(item_id: &str, encrypted_content: Option<&str>) -> Value {
    let mut fields = serde_json::Map::new();
    fields.insert("itemId".into(), Value::String(item_id.to_string()));
    if let Some(encrypted) = encrypted_content {
        fields.insert(
            "reasoningEncryptedContent".into(),
            Value::String(encrypted.to_string()),
        );
    }
    let mut namespaced = serde_json::Map::new();
    namespaced.insert(METADATA_NAMESPACE.to_string(), Value::Object(fields));
    Value::Object(namespaced)
}
