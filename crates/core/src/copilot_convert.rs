//! GitHub Copilot message conversion.
//!
//! Ports the observable behaviour of
//! `packages/core/src/github-copilot/chat/convert-to-openai-compatible-chat-messages.ts`:
//! convert canonical prompt messages into OpenAI-compatible chat messages,
//! including image URL parts, tool calls/results, and copilot reasoning fields.

use serde_json::{json, Map, Value};

use crate::{CoreError, CoreResult};

/// Convert canonical messages to OpenAI-compatible chat messages.
pub fn convert_to_openai_compatible_chat_messages(messages: &[Value]) -> CoreResult<Vec<Value>> {
    let mut output = Vec::new();
    for message in messages {
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("message missing role".into()))?;
        match role {
            "system" => output.push(json!({
                "role": "system",
                "content": content_text(message.get("content")),
            })),
            "user" => output.push(convert_user(message)),
            "assistant" => output.push(convert_assistant(message)),
            "tool" => output.push(convert_tool(message)),
            _ => {}
        }
    }
    Ok(output)
}

fn content_text(content: Option<&Value>) -> Value {
    match content {
        Some(Value::String(text)) => Value::String(text.clone()),
        Some(Value::Array(parts)) => {
            let joined: Vec<String> = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str).map(str::to_string))
                .collect();
            Value::String(joined.join(""))
        }
        _ => Value::Null,
    }
}

fn convert_user(message: &Value) -> Value {
    let parts: Vec<Value> = match message.get("content") {
        Some(Value::Array(parts)) => parts.iter().map(convert_part).collect(),
        _ => Vec::new(),
    };
    let content =
        if parts.len() == 1 && parts[0].get("type").and_then(Value::as_str) == Some("text") {
            parts[0].get("text").cloned().unwrap_or(Value::Null)
        } else {
            Value::Array(parts)
        };
    json!({ "role": "user", "content": content })
}

fn convert_assistant(message: &Value) -> Value {
    let mut text = Vec::new();
    let mut tool_calls = Vec::new();
    let mut reasoning_text: Option<String> = None;
    let mut reasoning_opaque: Option<String> = None;
    if let Some(Value::Array(parts)) = message.get("content") {
        for part in parts {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(value) = part.get("text").and_then(Value::as_str) {
                        text.push(value.to_string());
                    }
                }
                Some("reasoning") => {
                    reasoning_text = part.get("text").and_then(Value::as_str).map(str::to_string);
                    reasoning_opaque = part
                        .get("providerOptions")
                        .and_then(|options| options.get("copilot"))
                        .and_then(|copilot| copilot.get("reasoningOpaque"))
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }
                Some("tool-call") => {
                    let id = part.get("toolCallId").and_then(Value::as_str).unwrap_or("");
                    let name = part.get("toolName").and_then(Value::as_str).unwrap_or("");
                    let input = part.get("input").cloned().unwrap_or(Value::Null);
                    tool_calls.push(json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(&input).unwrap_or_default(),
                        },
                    }));
                }
                _ => {}
            }
        }
    }
    let mut object = Map::new();
    object.insert("role".into(), json!("assistant"));
    object.insert(
        "content".into(),
        if text.is_empty() {
            Value::Null
        } else {
            Value::String(text.join(""))
        },
    );
    if !tool_calls.is_empty() {
        object.insert("tool_calls".into(), Value::Array(tool_calls));
    }
    if let Some(reasoning_text) = reasoning_text {
        if let Some(reasoning_opaque) = reasoning_opaque {
            object.insert("reasoning_text".into(), Value::String(reasoning_text));
            object.insert("reasoning_opaque".into(), Value::String(reasoning_opaque));
        }
    }
    Value::Object(object)
}

fn convert_tool(message: &Value) -> Value {
    let mut tool_call_id = String::new();
    let mut content = String::new();
    if let Some(Value::Array(parts)) = message.get("content") {
        for part in parts {
            if part.get("type").and_then(Value::as_str) == Some("tool-result") {
                tool_call_id = part
                    .get("toolCallId")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                content = part
                    .get("output")
                    .and_then(|output| output.get("value"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    json!({ "role": "tool", "tool_call_id": tool_call_id, "content": content })
}

fn convert_part(part: &Value) -> Value {
    match part.get("type").and_then(Value::as_str) {
        Some("file") => {
            let data = part.get("data").and_then(Value::as_str).unwrap_or("");
            let media_type = part.get("mediaType").and_then(Value::as_str).unwrap_or("");
            let url = if data.starts_with("http://") || data.starts_with("https://") {
                data.to_string()
            } else {
                format!("data:{media_type};base64,{data}")
            };
            json!({ "type": "image_url", "image_url": { "url": url } })
        }
        Some("text") => {
            json!({ "type": "text", "text": part.get("text").cloned().unwrap_or(Value::Null) })
        }
        _ => part.clone(),
    }
}
