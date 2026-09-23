//! Session message projection and error serialization.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/opencode/test/session/message-v2.test.ts` (upstream 18ef3cc):
//! `MessageV2.toModelMessages` (the black-box subset) and `MessageV2.fromError`.
//!
//! Dropped (needs provider-specific wire transforms): the anthropic/bedrock
//! media re-placement, OpenRouter reasoning-details replay, signed-reasoning
//! empty-text substitution, and the `latest`/`filterCompacted` ordering cases.

use serde_json::{json, Map, Value};

/// A typed error serialization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for MessageError {}

const CONTEXT_OVERFLOW_MESSAGE: &str = "Input exceeds context window of this model";

const OVERFLOW_MARKERS: &[&str] = &[
    "prompt is too long",
    "exceeds the context window",
    "exceeds the maximum number of tokens allowed",
    "tokens in request more than max tokens allowed",
    "Please reduce the length of the messages or completion",
    "400 status code (no body)",
    "413 status code (no body)",
];

/// Serialize an arbitrary thrown value into the reference error envelope.
///
/// Mirrors `MessageV2.fromError`: context-overflow detection, provider
/// response-code mapping, retryable OpenAI stream errors, Zlib decompression
/// failures, and the unknown fallback.
pub fn from_error(input: &Value, _provider_id: &str) -> Value {
    if let Some(object) = input.as_object() {
        if let Some(response) = error_response(object) {
            return response;
        }
        if let Some(message) = object.get("message").and_then(Value::as_str) {
            let body = serde_json::to_string(input).unwrap_or_default();
            if let Some(response) = from_provider_body(message, object, &body) {
                return response;
            }
        }
    }
    json!({
        "name": "UnknownError",
        "data": { "message": stringify(input) },
    })
}

fn error_response(object: &Map<String, Value>) -> Option<Value> {
    if object.get("type").and_then(Value::as_str) != Some("error") {
        return None;
    }
    let error = object.get("error")?.as_object()?;
    let code = error.get("code").and_then(Value::as_str)?;
    let body = serde_json::to_string(object).unwrap_or_default();
    match code {
        "context_length_exceeded" => Some(context_overflow(&body)),
        "insufficient_quota" => Some(api_error(
            "Quota exceeded. Check your plan and billing details.",
            false,
            &body,
        )),
        "usage_not_included" => Some(api_error(
            "To use Codex with your ChatGPT plan, upgrade to Plus: https://chatgpt.com/explore/plus.",
            false,
            &body,
        )),
        "invalid_prompt" => Some(api_error(
            error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            false,
            &body,
        )),
        _ => None,
    }
}

fn from_provider_body(message: &str, object: &Map<String, Value>, body: &str) -> Option<Value> {
    if let Ok(parsed) = serde_json::from_str::<Value>(message) {
        if parsed
            .get("error")
            .and_then(|e| e.get("type"))
            .and_then(Value::as_str)
            == Some("server_error")
        {
            let inner = parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let response_body = serde_json::to_string(&parsed).unwrap_or_default();
            return Some(api_error(&inner, true, &response_body));
        }
    }

    if object.get("code").and_then(Value::as_str) == Some("ZlibError")
        || message.contains("ZlibError")
    {
        if object.get("aborted").and_then(Value::as_bool) == Some(true) {
            return Some(json!({
                "name": "MessageAbortedError",
                "data": { "message": "The operation was aborted" },
            }));
        }
        let text = "Response decompression failed".to_string();
        return Some(api_error(&text, true, body));
    }

    if OVERFLOW_MARKERS
        .iter()
        .any(|marker| message.contains(marker))
        || object
            .get("responseBody")
            .and_then(Value::as_str)
            .is_some_and(|response| response.contains("context_length_exceeded"))
    {
        return Some(context_overflow(body));
    }

    if object.get("statusCode").and_then(Value::as_u64).is_some() {
        return Some(api_error(message, false, body));
    }

    None
}

fn context_overflow(body: &str) -> Value {
    json!({
        "name": "ContextOverflowError",
        "data": {
            "message": CONTEXT_OVERFLOW_MESSAGE,
            "responseBody": body,
        },
    })
}

fn api_error(message: &str, retryable: bool, body: &str) -> Value {
    json!({
        "name": "APIError",
        "data": {
            "message": message,
            "isRetryable": retryable,
            "responseBody": body,
        },
    })
}

fn stringify(input: &Value) -> String {
    match input {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

/// Project a stored message list into provider model messages.
///
/// `model_provider_id`/`model_model_id` identify the target model; provider
/// metadata is only replayed when the assistant message was produced by the
/// same model. `tool_output_max_chars` truncates completed tool output.
pub fn to_model_messages(
    input: &Value,
    model_provider_id: &str,
    model_model_id: &str,
    tool_output_max_chars: Option<usize>,
) -> Value {
    let mut out: Vec<Value> = Vec::new();
    let Some(messages) = input.as_array() else {
        return Value::Array(out);
    };
    for message in messages {
        let info = &message["info"];
        let parts = message["parts"].as_array().cloned().unwrap_or_default();
        match info["role"].as_str() {
            Some("user") => {
                let content = project_user_parts(&parts);
                if !content.is_empty() {
                    out.push(json!({ "role": "user", "content": content }));
                }
            }
            Some("assistant") => {
                let matches = model_matches(info, model_provider_id, model_model_id);
                let aborted = info
                    .get("error")
                    .and_then(|error| error.get("name"))
                    .and_then(Value::as_str)
                    == Some("MessageAbortedError");
                if info.get("error").is_some() && !aborted {
                    continue;
                }
                for group in split_groups(&parts) {
                    if aborted && !group_has_content(&group) {
                        continue;
                    }
                    project_assistant_group(&group, matches, tool_output_max_chars, &mut out);
                }
            }
            _ => {}
        }
    }
    Value::Array(out)
}

fn project_user_parts(parts: &[Value]) -> Vec<Value> {
    let mut content = Vec::new();
    for part in parts {
        if part["ignored"].as_bool() == Some(true) {
            continue;
        }
        match part["type"].as_str() {
            Some("text") => {
                let text = part["text"].as_str().unwrap_or_default();
                if !text.is_empty() || part["synthetic"].as_bool() == Some(true) {
                    content.push(json!({ "type": "text", "text": text }));
                }
            }
            Some("file") => {
                let mime = part["mime"].as_str().unwrap_or_default();
                if mime.starts_with("image/") {
                    content.push(json!({
                        "type": "file",
                        "mediaType": mime,
                        "filename": part["filename"],
                        "data": part["url"],
                    }));
                }
            }
            Some("compaction") => {
                content.push(json!({ "type": "text", "text": "What did we do so far?" }));
            }
            Some("subtask") => {
                content.push(json!({
                    "type": "text",
                    "text": "The following tool was executed by the user",
                }));
            }
            _ => {}
        }
    }
    content
}

fn split_groups(parts: &[Value]) -> Vec<Vec<Value>> {
    let mut groups: Vec<Vec<Value>> = vec![Vec::new()];
    for part in parts {
        if part["type"].as_str() == Some("step-start") {
            let last_empty = match groups.last() {
                Some(group) => group.is_empty(),
                None => true,
            };
            if !last_empty {
                groups.push(Vec::new());
            }
            continue;
        }
        if let Some(group) = groups.last_mut() {
            group.push(part.clone());
        }
    }
    groups.retain(|group| !group.is_empty());
    groups
}

fn group_has_content(group: &[Value]) -> bool {
    group
        .iter()
        .any(|part| matches!(part["type"].as_str(), Some("text") | Some("tool")))
}

fn project_assistant_group(
    group: &[Value],
    matches: bool,
    tool_output_max_chars: Option<usize>,
    out: &mut Vec<Value>,
) {
    let mut content = Vec::new();
    let mut results = Vec::new();
    for part in group {
        match part["type"].as_str() {
            Some("text") => {
                let mut item =
                    json!({ "type": "text", "text": part["text"].as_str().unwrap_or_default() });
                attach_metadata(&mut item, part, matches);
                content.push(item);
            }
            Some("reasoning") => {
                if matches {
                    let mut item = json!({
                        "type": "reasoning",
                        "text": part["text"].as_str().unwrap_or_default(),
                    });
                    attach_metadata(&mut item, part, matches);
                    content.push(item);
                } else {
                    content.push(json!({
                        "type": "text",
                        "text": part["text"].as_str().unwrap_or_default(),
                    }));
                }
            }
            Some("tool") => {
                let call_id = part["callID"].clone();
                let tool_name = part["tool"].clone();
                let state = &part["state"];
                let mut call = json!({
                    "type": "tool-call",
                    "toolCallId": call_id,
                    "toolName": tool_name,
                    "input": state["input"],
                });
                attach_metadata(&mut call, part, matches);
                content.push(call);

                let mut result = json!({
                    "type": "tool-result",
                    "toolCallId": part["callID"],
                    "toolName": part["tool"],
                });
                result["output"] = tool_output(state, tool_output_max_chars);
                if matches && part.get("metadata").is_some() {
                    result["providerOptions"] = part["metadata"].clone();
                }
                results.push(result);
            }
            _ => {}
        }
    }
    if !content.is_empty() {
        out.push(json!({ "role": "assistant", "content": content }));
    }
    if !results.is_empty() {
        out.push(json!({ "role": "tool", "content": results }));
    }
}

fn attach_metadata(item: &mut Value, part: &Value, matches: bool) {
    if !matches {
        return;
    }
    if let Some(metadata) = part.get("metadata") {
        item["providerOptions"] = metadata.clone();
    }
}

fn tool_output(state: &Value, tool_output_max_chars: Option<usize>) -> Value {
    let status = state["status"].as_str().unwrap_or_default();
    match status {
        "completed" => {
            if state
                .get("time")
                .and_then(|time| time.get("compacted"))
                .is_some()
            {
                return json!({ "type": "text", "value": "[Old tool result content cleared]" });
            }
            let output = state["output"].as_str().unwrap_or_default();
            if let Some(max) = tool_output_max_chars {
                let chars: Vec<char> = output.chars().collect();
                if chars.len() > max {
                    let kept: String = chars[..max].iter().collect();
                    let omitted = chars.len() - max;
                    return json!({
                        "type": "text",
                        "value": format!("{kept}\n[Tool output truncated for compaction: omitted {omitted} chars]"),
                    });
                }
            }
            let attachments = state
                .get("attachments")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if attachments.is_empty() {
                return json!({ "type": "text", "value": output });
            }
            let mut value = vec![json!({ "type": "text", "text": output })];
            for attachment in &attachments {
                let mime = attachment["mime"].as_str().unwrap_or_default();
                let data = attachment["url"]
                    .as_str()
                    .and_then(|url| url.split_once("base64,").map(|(_, data)| data.to_string()))
                    .unwrap_or_default();
                value.push(json!({ "type": "media", "mediaType": mime, "data": data }));
            }
            json!({ "type": "content", "value": value })
        }
        "error" => {
            let metadata = &state["metadata"];
            if metadata.get("interrupted").and_then(Value::as_bool) == Some(true) {
                if let Some(output) = metadata.get("output").and_then(Value::as_str) {
                    return json!({ "type": "text", "value": output });
                }
            }
            json!({ "type": "error-text", "value": state["error"] })
        }
        _ => json!({ "type": "error-text", "value": "[Tool execution was interrupted]" }),
    }
}

fn model_matches(info: &Value, provider_id: &str, model_id: &str) -> bool {
    info["providerID"].as_str() == Some(provider_id) && info["modelID"].as_str() == Some(model_id)
}
