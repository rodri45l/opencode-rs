//! Session runner `toLLMMessages` (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/runner/to-llm-message.ts`: `toLLMMessages` omits
//! empty assistant turns, maps every top-level V2 Session message type, replays
//! durable tool media into canonical tool messages without structured base64,
//! restores OpenAI encrypted reasoning metadata, and drops provider-native
//! continuation metadata from failed assistant turns and after a model switch.
//! The `@opencode-ai/llm` `Message`/`Model` types are represented as JSON shapes;
//! no protocol runtime is exercised.

use serde_json::{json, Map, Value};

/// Convert V2 session messages into LLM messages for `model`.
pub fn to_llm_messages(messages: &[Value], model: &Value) -> Vec<Value> {
    let current_model = model.get("id").and_then(Value::as_str).unwrap_or("");
    let mut output = Vec::new();
    for message in messages {
        match message.get("type").and_then(Value::as_str) {
            Some("assistant") => {
                let (converted, tool_message) = convert_assistant(message, current_model);
                if let Some(converted) = converted {
                    output.push(converted);
                }
                if !tool_message.is_empty() {
                    output.push(json!({ "role": "tool", "content": tool_message }));
                }
            }
            Some("system") => {
                output.push(json!({
                    "role": "system",
                    "content": message.get("text").cloned().unwrap_or(Value::Null),
                }));
            }
            Some("user") => {
                let mut content = Vec::new();
                if let Some(text) = message.get("text").and_then(Value::as_str) {
                    content.push(json!({ "type": "text", "text": text }));
                }
                if let Some(files) = message.get("files").and_then(Value::as_array) {
                    for file in files {
                        content.push(json!({
                            "type": "media",
                            "mediaType": file.get("mime").cloned().unwrap_or(Value::Null),
                            "data": file.get("uri").cloned().unwrap_or(Value::Null),
                            "filename": file.get("name").cloned().unwrap_or(Value::Null),
                        }));
                    }
                }
                let mut converted = Map::new();
                converted.insert("role".into(), json!("user"));
                converted.insert("content".into(), Value::Array(content));
                if let Some(agents) = message.get("agents") {
                    converted.insert("metadata".into(), json!({ "agents": agents }));
                }
                output.push(Value::Object(converted));
            }
            Some("synthetic") => {
                output.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": message.get("text").cloned().unwrap_or(Value::Null) }],
                }));
            }
            Some("shell") => {
                let command = message.get("command").and_then(Value::as_str).unwrap_or("");
                let shell_output = message.get("output").and_then(Value::as_str).unwrap_or("");
                output.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": format!("Shell command: {command}\n\n{shell_output}") }],
                }));
            }
            Some("compaction") => {
                let summary = message.get("summary").and_then(Value::as_str).unwrap_or("");
                let recent = message.get("recent").and_then(Value::as_str).unwrap_or("");
                let checkpoint = format!(
                    "<conversation-checkpoint>\n<summary>\n{summary}\n</summary>\n<recent-context>\n{recent}\n</recent-context>\n</conversation-checkpoint>"
                );
                output.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": checkpoint }],
                }));
            }
            _ => {}
        }
    }
    output
}

fn convert_assistant(message: &Value, current_model: &str) -> (Option<Value>, Vec<Value>) {
    let assistant_model = message
        .get("model")
        .and_then(|model| model.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let switched = assistant_model != current_model;
    let failed = message.get("finish").and_then(Value::as_str) == Some("error")
        || message.get("error").is_some();

    let mut content = Vec::new();
    let mut tool_results = Vec::new();
    if let Some(parts) = message.get("content").and_then(Value::as_array) {
        for part in parts {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        if !text.is_empty() {
                            content.push(json!({ "type": "text", "text": text }));
                        }
                    }
                }
                Some("reasoning") => {
                    let text = part.get("text").and_then(Value::as_str).unwrap_or("");
                    let metadata = part.get("providerMetadata");
                    if switched {
                        if !text.is_empty() {
                            content.push(json!({ "type": "text", "text": text }));
                        }
                    } else if !text.is_empty() || metadata.is_some() {
                        let mut reasoning = Map::new();
                        reasoning.insert("type".into(), json!("reasoning"));
                        reasoning.insert("text".into(), json!(text));
                        if !failed {
                            if let Some(metadata) = metadata {
                                reasoning.insert("providerMetadata".into(), metadata.clone());
                            }
                        }
                        content.push(Value::Object(reasoning));
                    }
                }
                Some("tool") => {
                    let provider = part.get("provider");
                    let has_provider = provider.is_some();
                    let provider_executed = provider
                        .and_then(|provider| provider.get("executed"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let id = part.get("id").cloned().unwrap_or(Value::Null);
                    let name = part.get("name").cloned().unwrap_or(Value::Null);
                    let input = part
                        .get("state")
                        .and_then(|state| state.get("input"))
                        .map(parse_input)
                        .unwrap_or_else(|| json!({}));

                    let mut call = Map::new();
                    call.insert("type".into(), json!("tool-call"));
                    call.insert("id".into(), id.clone());
                    call.insert("name".into(), name.clone());
                    call.insert("input".into(), input);
                    if has_provider {
                        call.insert("providerExecuted".into(), json!(provider_executed));
                    }
                    if provider_executed && !failed && !switched {
                        if let Some(metadata) =
                            provider.and_then(|provider| provider.get("metadata"))
                        {
                            call.insert("providerMetadata".into(), metadata.clone());
                        }
                    }
                    content.push(Value::Object(call));

                    let status = part
                        .get("state")
                        .and_then(|state| state.get("status"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if status == "completed" || status == "error" {
                        let result = derive_result(part, provider_executed);
                        let mut result_object = Map::new();
                        result_object.insert("type".into(), json!("tool-result"));
                        result_object.insert("id".into(), id.clone());
                        result_object.insert("name".into(), name.clone());
                        if provider_executed {
                            result_object.insert("providerExecuted".into(), json!(true));
                            if !failed && !switched {
                                let provider = part.get("provider");
                                if let Some(metadata) = provider
                                    .and_then(|provider| provider.get("resultMetadata"))
                                    .or_else(|| {
                                        provider.and_then(|provider| provider.get("metadata"))
                                    })
                                {
                                    result_object
                                        .insert("providerMetadata".into(), metadata.clone());
                                }
                            }
                            result_object.insert("result".into(), result);
                            content.push(Value::Object(result_object));
                        } else {
                            result_object.insert("result".into(), result);
                            tool_results.push(Value::Object(result_object));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if content.is_empty() {
        return (None, tool_results);
    }
    let mut converted = Map::new();
    converted.insert(
        "id".into(),
        message.get("id").cloned().unwrap_or(Value::Null),
    );
    converted.insert("role".into(), json!("assistant"));
    converted.insert("content".into(), Value::Array(content));
    (Some(Value::Object(converted)), tool_results)
}

fn parse_input(input: &Value) -> Value {
    match input {
        Value::String(text) => serde_json::from_str(text).unwrap_or_else(|_| json!(text)),
        other => other.clone(),
    }
}

fn derive_result(part: &Value, provider_executed: bool) -> Value {
    let state = part.get("state").cloned().unwrap_or_else(|| json!({}));
    if let Some(result) = state.get("result") {
        return result.clone();
    }
    let content = state.get("content").cloned().unwrap_or_else(|| json!([]));
    let structured = state
        .get("structured")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if let Some(error) = state.get("error") {
        return json!({
            "type": "error",
            "value": { "error": error, "content": content, "structured": structured },
        });
    }
    let has_content = content
        .as_array()
        .map(|parts| !parts.is_empty())
        .unwrap_or(false);
    if provider_executed {
        if let Some(parts) = content.as_array() {
            if parts.len() == 1 && parts[0].get("type").and_then(Value::as_str) == Some("text") {
                return json!({ "type": "text", "value": parts[0].get("text").cloned().unwrap_or(Value::Null) });
            }
        }
    }
    if has_content {
        return json!({ "type": "content", "value": content });
    }
    json!({ "type": "json", "value": structured })
}
