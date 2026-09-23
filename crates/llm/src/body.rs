//! Protocol request-body lowering.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};
use crate::shared::ProviderShared;
use crate::tool_schema::ToolSchemaProjection;

const IMAGE_MIMES: &[&str] = &["image/png", "image/jpeg", "image/gif", "image/webp"];
const MEDIA_MIMES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "audio/wav",
    "audio/mp3",
    "audio/aiff",
    "audio/aac",
    "audio/ogg",
    "audio/flac",
];

fn arr(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

fn str_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn generation(request: &Value) -> Value {
    request
        .get("generation")
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn model_id(request: &Value) -> String {
    request
        .get("model")
        .and_then(|model| model.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn tool_schema_compat(request: &Value) -> Option<String> {
    request
        .get("model")
        .and_then(|model| model.get("compatibility"))
        .and_then(|compat| compat.get("toolSchema"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn base64_charset(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

fn validate_media(
    route: &str,
    part: &Value,
    supported: &[&str],
) -> LlmResult<(String, String, String)> {
    let mime = str_at(part, "mediaType").unwrap_or_default().to_lowercase();
    if !supported.contains(&mime.as_str()) {
        return Err(LlmError::InvalidRequest(format!(
            "{route} does not support media type {mime}"
        )));
    }
    let data = part.get("data").and_then(Value::as_str).unwrap_or("");
    let base64 = if let Some(rest) = data.strip_prefix("data:") {
        let Some((meta, payload)) = rest.split_once(',') else {
            return Err(LlmError::InvalidRequest(format!(
                "{route} media data URL must contain valid base64"
            )));
        };
        let declared = meta.split(';').next().unwrap_or("").to_lowercase();
        if declared != mime {
            return Err(LlmError::InvalidRequest(format!(
                "{route} media type {mime} does not match data URL type {declared}"
            )));
        }
        payload.to_string()
    } else {
        data.to_string()
    };
    if base64.len() > ProviderShared::MAX_MEDIA_ENCODED_BYTES {
        return Err(LlmError::InvalidRequest(format!(
            "{route} media exceeds the {} byte encoded limit",
            ProviderShared::MAX_MEDIA_ENCODED_BYTES
        )));
    }
    if base64.is_empty() || base64.len() % 4 != 0 || !base64_charset(&base64) {
        return Err(LlmError::InvalidRequest(format!(
            "{route} media must contain valid base64"
        )));
    }
    Ok((
        mime.clone(),
        base64.clone(),
        format!("data:{mime};base64,{base64}"),
    ))
}

fn cache_control(hint: &Value, breakpoints: &mut i32) -> Option<Value> {
    if *breakpoints <= 0 {
        return None;
    }
    *breakpoints -= 1;
    let mut out = Map::new();
    out.insert("type".into(), Value::String("ephemeral".into()));
    if let Some(ttl) = hint.get("ttlSeconds").and_then(Value::as_i64) {
        if ttl >= 3600 {
            out.insert("ttl".into(), Value::String("1h".into()));
        }
    }
    Some(Value::Object(out))
}

fn apply_cache(source: &Value, target: &mut Value, breakpoints: &mut i32) {
    if let Some(hint) = source.get("cache").cloned() {
        if let Some(control) = cache_control(&hint, breakpoints) {
            target["cache_control"] = control;
        }
    }
}

/// Build the protocol-native body for a route.
pub fn build(protocol: &str, request: &Value) -> LlmResult<Value> {
    match protocol {
        "openai-chat"
        | "openai-compatible-chat"
        | "openrouter"
        | "cloudflare-ai-gateway"
        | "cloudflare-workers-ai"
        | "openai-chat-compatible" => openai_chat(request),
        "anthropic-messages" => anthropic_messages(request),
        "gemini" => gemini(request),
        "bedrock-converse" => bedrock_converse(request),
        "openai-responses" | "openai-responses-websocket" => openai_responses(request),
        other => Err(LlmError::InvalidRequest(format!(
            "unknown protocol {other}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// OpenAI Chat
// ---------------------------------------------------------------------------

fn openai_tool_choice(choice: &Value) -> LlmResult<Value> {
    match choice.get("type").and_then(Value::as_str) {
        Some("auto") => Ok(json!("auto")),
        Some("none") => Ok(json!("none")),
        Some("required") => Ok(json!("required")),
        Some("tool") => {
            let name = choice.get("name").and_then(Value::as_str).unwrap_or("");
            Ok(json!({ "type": "function", "function": { "name": name } }))
        }
        _ => Err(LlmError::InvalidRequest("unknown tool choice".into())),
    }
}

fn openai_user_message(message: &Value) -> LlmResult<Value> {
    let mut content = Vec::new();
    for part in arr(message.get("content").unwrap_or(&Value::Null)) {
        match part.get("type").and_then(Value::as_str) {
            Some("text") => content.push(json!({ "type": "text", "text": part["text"] })),
            Some("media") => {
                let (_, _, url) = validate_media("OpenAI Chat", &part, IMAGE_MIMES)?;
                content.push(json!({ "type": "image_url", "image_url": { "url": url } }));
            }
            _ => {
                return Err(LlmError::InvalidRequest(
                    "OpenAI Chat user messages only support text and media content for now".into(),
                ))
            }
        }
    }
    if content.iter().all(|part| part["type"] == "text") {
        let text = content
            .iter()
            .map(|part| part["text"].as_str().unwrap_or(""))
            .collect::<String>();
        return Ok(json!({ "role": "user", "content": text }));
    }
    Ok(json!({ "role": "user", "content": content }))
}

fn openai_assistant_message(message: &Value) -> LlmResult<Value> {
    let mut text = Vec::new();
    let mut reasoning = Vec::new();
    let mut tool_calls = Vec::new();
    for part in arr(message.get("content").unwrap_or(&Value::Null)) {
        match part.get("type").and_then(Value::as_str) {
            Some("text") => text.push(part.clone()),
            Some("reasoning") => reasoning.push(part.clone()),
            Some("tool-call") => tool_calls.push(json!({
                "id": part["id"],
                "type": "function",
                "function": {
                    "name": part["name"],
                    "arguments": ProviderShared::encode_json(part.get("input").unwrap_or(&Value::Null)),
                }
            })),
            _ => {
                return Err(LlmError::InvalidRequest(
                    "OpenAI Chat assistant messages only support text, reasoning, and tool-call content for now"
                        .into(),
                ))
            }
        }
    }
    let mut out = Map::new();
    out.insert("role".into(), Value::String("assistant".into()));
    out.insert(
        "content".into(),
        if text.is_empty() {
            Value::Null
        } else {
            Value::String(ProviderShared::join_text(&text))
        },
    );
    if !tool_calls.is_empty() {
        out.insert("tool_calls".into(), Value::Array(tool_calls));
    }
    if !reasoning.is_empty() {
        let joined = reasoning
            .iter()
            .map(|part| part["text"].as_str().unwrap_or(""))
            .collect::<String>();
        out.insert("reasoning_content".into(), Value::String(joined));
    }
    Ok(Value::Object(out))
}

fn openai_tool_messages(message: &Value) -> LlmResult<(Vec<Value>, Vec<Value>)> {
    let mut messages = Vec::new();
    let mut images = Vec::new();
    for part in arr(message.get("content").unwrap_or(&Value::Null)) {
        if part.get("type").and_then(Value::as_str) != Some("tool-result") {
            return Err(LlmError::InvalidRequest(
                "OpenAI Chat tool messages only support tool-result content for now".into(),
            ));
        }
        let result = part.get("result").cloned().unwrap_or(Value::Null);
        if result.get("type").and_then(Value::as_str) != Some("content") {
            messages.push(json!({
                "role": "tool",
                "tool_call_id": part["id"],
                "content": ProviderShared::tool_result_text(&part),
            }));
            continue;
        }
        let content = arr(result.get("value").unwrap_or(&Value::Null));
        let text = content
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("text"))
            .map(|item| item["text"].as_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        messages.push(json!({
            "role": "tool",
            "tool_call_id": part["id"],
            "content": text,
        }));
        for item in content
            .iter()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("file"))
        {
            let (_, _, url) = validate_media(
                "OpenAI Chat",
                &json!({ "mediaType": item["mime"], "data": item["uri"] }),
                IMAGE_MIMES,
            )?;
            images.push(json!({ "type": "image_url", "image_url": { "url": url } }));
        }
    }
    Ok((messages, images))
}

fn openai_messages(request: &Value) -> LlmResult<Vec<Value>> {
    let mut messages = Vec::new();
    let system = arr(request.get("system").unwrap_or(&Value::Null));
    if !system.is_empty() {
        messages.push(json!({ "role": "system", "content": ProviderShared::join_text(&system) }));
    }
    let mut pending_images: Vec<Value> = Vec::new();
    let flush = |messages: &mut Vec<Value>, pending: &mut Vec<Value>| {
        if !pending.is_empty() {
            messages.push(json!({ "role": "user", "content": pending.split_off(0) }));
        }
    };
    for message in arr(request.get("messages").unwrap_or(&Value::Null)) {
        match message.get("role").and_then(Value::as_str) {
            Some("system") => {
                let text = ProviderShared::wrap_system_update(&arr(message
                    .get("content")
                    .unwrap_or(&Value::Null)));
                if !pending_images.is_empty() {
                    let mut content = pending_images.split_off(0);
                    content.push(json!({ "type": "text", "text": text }));
                    messages.push(json!({ "role": "user", "content": content }));
                    continue;
                }
                let previous = messages.last().cloned();
                match previous {
                    Some(prev) if prev["role"] == "user" && prev["content"].is_string() => {
                        let existing = prev["content"].as_str().unwrap_or("");
                        messages.last_mut().unwrap()["content"] =
                            Value::String(format!("{existing}\n{text}"));
                    }
                    Some(prev) if prev["role"] == "user" && prev["content"].is_array() => {
                        if let Some(content) =
                            messages.last_mut().unwrap()["content"].as_array_mut()
                        {
                            content.push(json!({ "type": "text", "text": text }));
                        }
                    }
                    _ => messages.push(json!({ "role": "user", "content": text })),
                }
            }
            Some("tool") => {
                let (lowered, images) = openai_tool_messages(&message)?;
                messages.extend(lowered);
                pending_images.extend(images);
            }
            Some("user") => {
                flush(&mut messages, &mut pending_images);
                messages.push(openai_user_message(&message)?);
            }
            Some("assistant") => {
                flush(&mut messages, &mut pending_images);
                messages.push(openai_assistant_message(&message)?);
            }
            _ => {}
        }
    }
    flush(&mut messages, &mut pending_images);
    Ok(messages)
}

fn openai_chat(request: &Value) -> LlmResult<Value> {
    let generation = generation(request);
    let mut body = Map::new();
    body.insert("model".into(), Value::String(model_id(request)));
    body.insert("messages".into(), Value::Array(openai_messages(request)?));

    let tools = arr(request.get("tools").unwrap_or(&Value::Null));
    if !tools.is_empty() {
        let compat = tool_schema_compat(request);
        let lowered: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let schema = tool
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let schema = ToolSchemaProjection::model_compatibility(schema, compat.as_deref());
                let schema = ToolSchemaProjection::open_ai(schema.clone()).unwrap_or(schema);
                json!({
                    "type": "function",
                    "function": {
                        "name": tool["name"],
                        "description": tool.get("description").cloned().unwrap_or(Value::Null),
                        "parameters": schema,
                    }
                })
            })
            .collect();
        body.insert("tools".into(), Value::Array(lowered));
    }

    if let Some(choice) = request.get("toolChoice") {
        body.insert("tool_choice".into(), openai_tool_choice(choice)?);
    }

    body.insert("stream".into(), Value::Bool(true));
    body.insert("stream_options".into(), json!({ "include_usage": true }));

    if let Some(options) = request
        .get("providerOptions")
        .and_then(|options| options.get("openai"))
        .and_then(Value::as_object)
    {
        let store = options
            .get("store")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        body.insert("store".into(), Value::Bool(store));
        if let Some(effort) = options.get("reasoningEffort").and_then(Value::as_str) {
            body.insert("reasoning_effort".into(), Value::String(effort.into()));
        }
    }

    if let Some(options) = request
        .get("providerOptions")
        .and_then(|options| options.get("openrouter"))
        .and_then(Value::as_object)
    {
        if options.get("usage").and_then(Value::as_bool) == Some(true) {
            body.insert("usage".into(), json!({ "include": true }));
        }
        if let Some(reasoning) = options.get("reasoning").filter(|value| !value.is_null()) {
            body.insert("reasoning".into(), reasoning.clone());
        }
        if let Some(key) = options.get("promptCacheKey").and_then(Value::as_str) {
            body.insert("prompt_cache_key".into(), Value::String(key.into()));
        }
    }

    insert_generation_openai(&mut body, &generation);
    Ok(Value::Object(body))
}

fn insert_generation_openai(body: &mut Map<String, Value>, generation: &Value) {
    let mapping = [
        ("maxTokens", "max_tokens"),
        ("temperature", "temperature"),
        ("topP", "top_p"),
        ("frequencyPenalty", "frequency_penalty"),
        ("presencePenalty", "presence_penalty"),
        ("seed", "seed"),
        ("stop", "stop"),
    ];
    for (source, target) in mapping {
        if let Some(value) = generation.get(source).filter(|value| !value.is_null()) {
            body.insert(target.into(), value.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Anthropic Messages
// ---------------------------------------------------------------------------

fn anthropic_supports_native_system(id: &str) -> bool {
    id.contains("opus-4-8") || id.contains("claude-opus-4")
}

fn anthropic_tool_choice(choice: &Value) -> Value {
    match choice.get("type").and_then(Value::as_str) {
        Some("auto") => json!({ "type": "auto" }),
        Some("none") => json!({ "type": "none" }),
        Some("required") => json!({ "type": "any" }),
        Some("tool") => json!({ "type": "tool", "name": choice["name"] }),
        _ => json!({ "type": "auto" }),
    }
}

fn anthropic_content_parts(message: &Value, breakpoints: &mut i32) -> LlmResult<Vec<Value>> {
    let mut parts = Vec::new();
    for part in arr(message.get("content").unwrap_or(&Value::Null)) {
        match part.get("type").and_then(Value::as_str) {
            Some("text") => {
                let mut out = json!({ "type": "text", "text": part["text"] });
                apply_cache(&part, &mut out, breakpoints);
                parts.push(out);
            }
            Some("media") => {
                let (mime, data, _) = validate_media("Anthropic", &part, IMAGE_MIMES)?;
                parts.push(json!({
                    "type": "image",
                    "source": { "type": "base64", "media_type": mime, "data": data }
                }));
            }
            Some("reasoning") => {
                parts.push(json!({ "type": "text", "text": part["text"] }));
            }
            Some("tool-call") => {
                parts.push(json!({
                    "type": "tool_use",
                    "id": part["id"],
                    "name": part["name"],
                    "input": part.get("input").cloned().unwrap_or(Value::Null),
                }));
            }
            _ => {
                return Err(LlmError::InvalidRequest(
                    "Anthropic assistant messages only support text, reasoning, and tool-call content for now"
                        .into(),
                ))
            }
        }
    }
    Ok(parts)
}

fn anthropic_messages(request: &Value) -> LlmResult<Value> {
    let generation = generation(request);
    let id = model_id(request);
    let mut breakpoints = 4i32;
    let mut body = Map::new();
    body.insert("model".into(), Value::String(id.clone()));

    let system = arr(request.get("system").unwrap_or(&Value::Null));
    if !system.is_empty() {
        let mut parts = Vec::new();
        for part in &system {
            let mut out = json!({ "type": "text", "text": part["text"] });
            apply_cache(part, &mut out, &mut breakpoints);
            parts.push(out);
        }
        body.insert("system".into(), Value::Array(parts));
    }

    let mut messages: Vec<Value> = Vec::new();
    for message in arr(request.get("messages").unwrap_or(&Value::Null)) {
        match message.get("role").and_then(Value::as_str) {
            Some("system") => {
                if anthropic_supports_native_system(&id) {
                    let content = anthropic_content_parts(&message, &mut breakpoints)?;
                    messages.push(json!({ "role": "system", "content": content }));
                } else {
                    let text = ProviderShared::wrap_system_update(&arr(message
                        .get("content")
                        .unwrap_or(&Value::Null)));
                    let appended = messages
                        .last_mut()
                        .filter(|prev| prev["role"] == "user")
                        .and_then(|prev| prev.get_mut("content"))
                        .and_then(Value::as_array_mut);
                    match appended {
                        Some(content) => content.push(json!({ "type": "text", "text": text })),
                        None => messages.push(json!({
                            "role": "user",
                            "content": [{ "type": "text", "text": text }]
                        })),
                    }
                }
            }
            Some("user") | Some("assistant") => {
                let role = message["role"].clone();
                let content = anthropic_content_parts(&message, &mut breakpoints)?;
                messages.push(json!({ "role": role, "content": content }));
            }
            Some("tool") => {
                let mut content = Vec::new();
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    let result = part.get("result").cloned().unwrap_or(Value::Null);
                    if result.get("type").and_then(Value::as_str) == Some("content") {
                        let mut blocks = Vec::new();
                        for item in arr(result.get("value").unwrap_or(&Value::Null)) {
                            match item.get("type").and_then(Value::as_str) {
                                Some("text") => {
                                    blocks.push(json!({ "type": "text", "text": item["text"] }))
                                }
                                Some("file") => {
                                    let (mime, data, _) = validate_media(
                                        "Anthropic",
                                        &json!({ "mediaType": item["mime"], "data": item["uri"] }),
                                        IMAGE_MIMES,
                                    )?;
                                    blocks.push(json!({
                                        "type": "image",
                                        "source": { "type": "base64", "media_type": mime, "data": data }
                                    }));
                                }
                                _ => {}
                            }
                        }
                        content.push(json!({
                            "type": "tool_result",
                            "tool_use_id": part["id"],
                            "content": blocks,
                        }));
                    } else {
                        content.push(json!({
                            "type": "tool_result",
                            "tool_use_id": part["id"],
                            "content": ProviderShared::tool_result_text(&part),
                        }));
                    }
                }
                messages.push(json!({ "role": "user", "content": content }));
            }
            _ => {}
        }
    }
    body.insert("messages".into(), Value::Array(messages));

    let tools = arr(request.get("tools").unwrap_or(&Value::Null));
    if !tools.is_empty() {
        let lowered: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let mut out = json!({
                    "name": tool["name"],
                    "description": tool.get("description").cloned().unwrap_or(Value::Null),
                    "input_schema": tool.get("inputSchema").cloned().unwrap_or_else(|| json!({})),
                });
                apply_cache(tool, &mut out, &mut breakpoints);
                out
            })
            .collect();
        body.insert("tools".into(), Value::Array(lowered));
    }

    if let Some(choice) = request.get("toolChoice") {
        body.insert("tool_choice".into(), anthropic_tool_choice(choice));
    }

    body.insert("stream".into(), Value::Bool(true));
    let max_tokens = generation
        .get("maxTokens")
        .and_then(Value::as_i64)
        .or_else(|| {
            request
                .get("model")
                .and_then(|model| model.get("defaults"))
                .and_then(|defaults| defaults.get("limits"))
                .and_then(|limits| limits.get("output"))
                .and_then(Value::as_i64)
        })
        .unwrap_or(4096);
    body.insert("max_tokens".into(), json!(max_tokens));
    if let Some(temperature) = generation.get("temperature").filter(|v| !v.is_null()) {
        body.insert("temperature".into(), temperature.clone());
    }
    if let Some(top_p) = generation.get("topP").filter(|v| !v.is_null()) {
        body.insert("top_p".into(), top_p.clone());
    }
    if let Some(top_k) = generation.get("topK").filter(|v| !v.is_null()) {
        body.insert("top_k".into(), top_k.clone());
    }
    if let Some(stop) = generation.get("stop").filter(|v| !v.is_null()) {
        body.insert("stop_sequences".into(), stop.clone());
    }
    Ok(Value::Object(body))
}

// ---------------------------------------------------------------------------
// Gemini
// ---------------------------------------------------------------------------

fn gemini_tool_choice(choice: &Value) -> Value {
    match choice.get("type").and_then(Value::as_str) {
        Some("auto") => json!({ "functionCallingConfig": { "mode": "AUTO" } }),
        Some("none") => json!({ "functionCallingConfig": { "mode": "NONE" } }),
        Some("required") => json!({ "functionCallingConfig": { "mode": "ANY" } }),
        Some("tool") => json!({
            "functionCallingConfig": { "mode": "ANY", "allowedFunctionNames": [choice["name"]] }
        }),
        _ => json!({ "functionCallingConfig": { "mode": "AUTO" } }),
    }
}

fn gemini(request: &Value) -> LlmResult<Value> {
    let generation = generation(request);
    let mut body = Map::new();
    let mut contents: Vec<Value> = Vec::new();

    for message in arr(request.get("messages").unwrap_or(&Value::Null)) {
        match message.get("role").and_then(Value::as_str) {
            Some("system") => {
                let text = ProviderShared::wrap_system_update(&arr(message
                    .get("content")
                    .unwrap_or(&Value::Null)));
                match contents
                    .last_mut()
                    .filter(|prev| prev["role"] == "user")
                    .and_then(|prev| prev.get_mut("parts"))
                    .and_then(Value::as_array_mut)
                {
                    Some(parts) => parts.push(json!({ "text": text })),
                    None => contents.push(json!({ "role": "user", "parts": [{ "text": text }] })),
                }
            }
            Some("user") => {
                let mut parts = Vec::new();
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    match part.get("type").and_then(Value::as_str) {
                        Some("text") => parts.push(json!({ "text": part["text"] })),
                        Some("media") => {
                            let (mime, data, _) = validate_media("Gemini", &part, MEDIA_MIMES)?;
                            parts.push(json!({ "inlineData": { "mimeType": mime, "data": data } }));
                        }
                        _ => {
                            return Err(LlmError::InvalidRequest(
                                "Gemini user messages only support text and media content for now"
                                    .into(),
                            ))
                        }
                    }
                }
                contents.push(json!({ "role": "user", "parts": parts }));
            }
            Some("assistant") => {
                let mut parts = Vec::new();
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    match part.get("type").and_then(Value::as_str) {
                        Some("text") => parts.push(json!({ "text": part["text"] })),
                        Some("reasoning") => parts.push(json!({
                            "text": part["text"],
                            "thought": true,
                        })),
                        Some("tool-call") => parts.push(json!({
                            "functionCall": {
                                "name": part["name"],
                                "args": part.get("input").cloned().unwrap_or(Value::Null),
                            }
                        })),
                        _ => {
                            return Err(LlmError::InvalidRequest(
                                "Gemini assistant messages only support text, reasoning, and tool-call content for now"
                                    .into(),
                            ))
                        }
                    }
                }
                contents.push(json!({ "role": "model", "parts": parts }));
            }
            Some("tool") => {
                let mut parts = Vec::new();
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    let result = part.get("result").cloned().unwrap_or(Value::Null);
                    let mut response = Map::new();
                    response.insert("name".into(), part["name"].clone());
                    let mut images: Vec<Value> = Vec::new();
                    if result.get("type").and_then(Value::as_str) == Some("content") {
                        let mut text = Vec::new();
                        for item in arr(result.get("value").unwrap_or(&Value::Null)) {
                            match item.get("type").and_then(Value::as_str) {
                                Some("text") => {
                                    text.push(item["text"].as_str().unwrap_or("").to_string())
                                }
                                Some("file") => {
                                    let (mime, data, _) = validate_media(
                                        "Gemini",
                                        &json!({ "mediaType": item["mime"], "data": item["uri"] }),
                                        MEDIA_MIMES,
                                    )?;
                                    images.push(
                                        json!({ "inlineData": { "mimeType": mime, "data": data } }),
                                    );
                                }
                                _ => {}
                            }
                        }
                        response.insert("content".into(), Value::String(text.join("\n")));
                    } else {
                        response.insert(
                            "content".into(),
                            Value::String(ProviderShared::tool_result_text(&part)),
                        );
                    }
                    parts.push(json!({
                        "functionResponse": {
                            "name": part["name"],
                            "response": Value::Object(response),
                        }
                    }));
                    parts.extend(images);
                }
                contents.push(json!({ "role": "user", "parts": parts }));
            }
            _ => {}
        }
    }

    body.insert("contents".into(), Value::Array(contents));

    let system = arr(request.get("system").unwrap_or(&Value::Null));
    if !system.is_empty() {
        body.insert(
            "systemInstruction".into(),
            json!({ "parts": system.iter().map(|part| json!({ "text": part["text"] })).collect::<Vec<_>>() }),
        );
    }

    let tools = arr(request.get("tools").unwrap_or(&Value::Null));
    let choice_is_none = request
        .get("toolChoice")
        .and_then(|choice| choice.get("type"))
        .and_then(Value::as_str)
        == Some("none");
    if !tools.is_empty() && !choice_is_none {
        let declarations: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let schema = tool
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let schema = ToolSchemaProjection::gemini(schema.clone()).unwrap_or(schema);
                json!({
                    "name": tool["name"],
                    "description": tool.get("description").cloned().unwrap_or(Value::Null),
                    "parameters": schema,
                })
            })
            .collect();
        body.insert(
            "tools".into(),
            json!([{ "functionDeclarations": declarations }]),
        );
    }

    if let Some(choice) = request.get("toolChoice") {
        if choice.get("type").and_then(Value::as_str) != Some("none") {
            body.insert("toolConfig".into(), gemini_tool_choice(choice));
        }
    }

    let mut config = Map::new();
    if let Some(value) = generation.get("maxTokens").filter(|v| !v.is_null()) {
        config.insert("maxOutputTokens".into(), value.clone());
    }
    if let Some(value) = generation.get("temperature").filter(|v| !v.is_null()) {
        config.insert("temperature".into(), value.clone());
    }
    if let Some(value) = generation.get("topP").filter(|v| !v.is_null()) {
        config.insert("topP".into(), value.clone());
    }
    if let Some(value) = generation.get("topK").filter(|v| !v.is_null()) {
        config.insert("topK".into(), value.clone());
    }
    if let Some(value) = generation.get("stop").filter(|v| !v.is_null()) {
        config.insert("stopSequences".into(), value.clone());
    }
    if !config.is_empty() {
        body.insert("generationConfig".into(), Value::Object(config));
    }

    Ok(Value::Object(body))
}

// ---------------------------------------------------------------------------
// Bedrock Converse
// ---------------------------------------------------------------------------

fn bedrock_tool_choice(choice: &Value) -> Value {
    match choice.get("type").and_then(Value::as_str) {
        Some("auto") => json!({ "auto": {} }),
        Some("none") => json!({ "none": {} }),
        Some("required") => json!({ "any": {} }),
        Some("tool") => json!({ "tool": { "name": choice["name"] } }),
        _ => json!({ "auto": {} }),
    }
}

fn bedrock_converse(request: &Value) -> LlmResult<Value> {
    let generation = generation(request);
    let mut body = Map::new();
    body.insert("modelId".into(), Value::String(model_id(request)));
    let mut breakpoints = 4i32;

    let system = arr(request.get("system").unwrap_or(&Value::Null));
    if !system.is_empty() {
        let mut parts: Vec<Value> = Vec::new();
        for part in &system {
            parts.push(json!({ "text": part["text"] }));
            if part.get("cache").is_some() && breakpoints > 0 {
                breakpoints -= 1;
                parts.push(json!({ "cachePoint": { "type": "default" } }));
            }
        }
        body.insert("system".into(), Value::Array(parts));
    }

    let mut messages: Vec<Value> = Vec::new();
    for message in arr(request.get("messages").unwrap_or(&Value::Null)) {
        let role = message["role"].as_str().unwrap_or("");
        let bedrock_role = if role == "assistant" {
            "assistant"
        } else {
            "user"
        };
        let mut content: Vec<Value> = Vec::new();
        for part in arr(message.get("content").unwrap_or(&Value::Null)) {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => content.push(json!({ "text": part["text"] })),
                Some("media") => {
                    let mime = str_at(&part, "mediaType")
                        .unwrap_or_default()
                        .to_lowercase();
                    let format = match mime.as_str() {
                        "image/png" => Some("png"),
                        "image/jpeg" | "image/jpg" => Some("jpeg"),
                        "image/webp" => Some("webp"),
                        "image/gif" => Some("gif"),
                        other if other.starts_with("image/") => {
                            return Err(LlmError::InvalidRequest(format!(
                                "Bedrock Converse does not support image media type {other}"
                            )))
                        }
                        other => {
                            return Err(LlmError::InvalidRequest(format!(
                                "Bedrock Converse does not support media type {other}"
                            )))
                        }
                    };
                    let data = part.get("data").and_then(Value::as_str).unwrap_or("");
                    let bytes = data.rsplit(',').next().unwrap_or(data).to_string();
                    content.push(json!({
                        "image": { "format": format, "source": { "bytes": bytes } }
                    }));
                }
                Some("tool-call") => content.push(json!({
                    "toolUse": {
                        "toolUseId": part["id"],
                        "name": part["name"],
                        "input": part.get("input").cloned().unwrap_or(Value::Null),
                    }
                })),
                Some("tool-result") => {
                    let result = part.get("result").cloned().unwrap_or(Value::Null);
                    let payload = if result.get("type").and_then(Value::as_str) == Some("json") {
                        json!({ "json": result.get("value").cloned().unwrap_or(Value::Null) })
                    } else {
                        json!({ "text": ProviderShared::tool_result_text(&part) })
                    };
                    content.push(json!({
                        "toolResult": {
                            "toolUseId": part["id"],
                            "content": [payload],
                            "status": "success",
                        }
                    }));
                }
                _ => {}
            }
            if part.get("cache").is_some() && breakpoints > 0 {
                breakpoints -= 1;
                content.push(json!({ "cachePoint": { "type": "default" } }));
            }
        }
        messages.push(json!({ "role": bedrock_role, "content": content }));
    }
    body.insert("messages".into(), Value::Array(messages));

    let mut inference = Map::new();
    if let Some(value) = generation.get("maxTokens").filter(|v| !v.is_null()) {
        inference.insert("maxTokens".into(), value.clone());
    }
    if let Some(value) = generation.get("temperature").filter(|v| !v.is_null()) {
        inference.insert("temperature".into(), value.clone());
    }
    if let Some(value) = generation.get("topP").filter(|v| !v.is_null()) {
        inference.insert("topP".into(), value.clone());
    }
    if let Some(value) = generation.get("stop").filter(|v| !v.is_null()) {
        inference.insert("stopSequences".into(), value.clone());
    }
    if !inference.is_empty() {
        body.insert("inferenceConfig".into(), Value::Object(inference));
    }
    if let Some(value) = generation.get("topK").filter(|v| !v.is_null()) {
        body.insert(
            "additionalModelRequestFields".into(),
            json!({ "top_k": value }),
        );
    }

    let tools = arr(request.get("tools").unwrap_or(&Value::Null));
    if !tools.is_empty() {
        let mut lowered: Vec<Value> = tools
            .iter()
            .map(|tool| {
                json!({ "toolSpec": {
                    "name": tool["name"],
                    "description": tool.get("description").cloned().unwrap_or(Value::Null),
                    "inputSchema": { "json": tool.get("inputSchema").cloned().unwrap_or_else(|| json!({})) },
                } })
            })
            .collect();
        for tool in &tools {
            if tool.get("cache").is_some() && breakpoints > 0 {
                breakpoints -= 1;
                lowered.push(json!({ "cachePoint": { "type": "default" } }));
            }
        }
        let mut tool_config = Map::new();
        tool_config.insert("tools".into(), Value::Array(lowered));
        if let Some(choice) = request.get("toolChoice") {
            tool_config.insert("toolChoice".into(), bedrock_tool_choice(choice));
        }
        body.insert("toolConfig".into(), Value::Object(tool_config));
    }

    Ok(Value::Object(body))
}

// ---------------------------------------------------------------------------
// OpenAI Responses
// ---------------------------------------------------------------------------

fn openai_responses_tool_choice(choice: &Value) -> LlmResult<Value> {
    match choice.get("type").and_then(Value::as_str) {
        Some("auto") => Ok(json!("auto")),
        Some("none") => Ok(json!("none")),
        Some("required") => Ok(json!("required")),
        Some("tool") => {
            let name = choice.get("name").and_then(Value::as_str).unwrap_or("");
            Ok(json!({ "type": "function", "name": name }))
        }
        _ => Err(LlmError::InvalidRequest("unknown tool choice".into())),
    }
}

fn openai_responses(request: &Value) -> LlmResult<Value> {
    let generation = generation(request);
    let mut body = Map::new();
    body.insert("model".into(), Value::String(model_id(request)));

    let mut input: Vec<Value> = Vec::new();
    let system = arr(request.get("system").unwrap_or(&Value::Null));
    if !system.is_empty() {
        input.push(json!({
            "role": "system",
            "content": ProviderShared::join_text(&system),
        }));
    }
    for message in arr(request.get("messages").unwrap_or(&Value::Null)) {
        match message.get("role").and_then(Value::as_str) {
            Some("system") => {
                let text = ProviderShared::wrap_system_update(&arr(message
                    .get("content")
                    .unwrap_or(&Value::Null)));
                let appended = input
                    .last_mut()
                    .filter(|prev| prev["role"] == "user")
                    .and_then(|prev| prev.get_mut("content"))
                    .and_then(Value::as_array_mut);
                match appended {
                    Some(content) => content.push(json!({ "type": "input_text", "text": text })),
                    None => input.push(json!({
                        "role": "user",
                        "content": [{ "type": "input_text", "text": text }]
                    })),
                }
            }
            Some("user") => {
                let content: Vec<Value> = arr(message.get("content").unwrap_or(&Value::Null))
                    .iter()
                    .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
                    .map(|part| json!({ "type": "input_text", "text": part["text"] }))
                    .collect();
                input.push(json!({ "role": "user", "content": content }));
            }
            Some("assistant") => {
                let mut content: Vec<Value> = Vec::new();
                let mut calls: Vec<Value> = Vec::new();
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    match part.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            content.push(json!({ "type": "output_text", "text": part["text"] }))
                        }
                        Some("tool-call") => calls.push(json!({
                            "type": "function_call",
                            "call_id": part["id"],
                            "name": part["name"],
                            "arguments": ProviderShared::encode_json(part.get("input").unwrap_or(&Value::Null)),
                        })),
                        _ => {}
                    }
                }
                if !content.is_empty() {
                    input.push(json!({ "role": "assistant", "content": content }));
                }
                input.extend(calls);
            }
            Some("tool") => {
                for part in arr(message.get("content").unwrap_or(&Value::Null)) {
                    input.push(json!({
                        "type": "function_call_output",
                        "call_id": part["id"],
                        "output": ProviderShared::tool_result_text(&part),
                    }));
                }
            }
            _ => {}
        }
    }
    body.insert("input".into(), Value::Array(input));

    let tools = arr(request.get("tools").unwrap_or(&Value::Null));
    if !tools.is_empty() {
        let compat = tool_schema_compat(request);
        let lowered: Vec<Value> = tools
            .iter()
            .map(|tool| {
                let schema = tool
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let schema = ToolSchemaProjection::model_compatibility(schema, compat.as_deref());
                let schema = ToolSchemaProjection::open_ai(schema.clone()).unwrap_or(schema);
                json!({
                    "type": "function",
                    "name": tool["name"],
                    "description": tool.get("description").cloned().unwrap_or(Value::Null),
                    "strict": false,
                    "parameters": schema,
                })
            })
            .collect();
        body.insert("tools".into(), Value::Array(lowered));
    }
    if let Some(choice) = request.get("toolChoice") {
        body.insert("tool_choice".into(), openai_responses_tool_choice(choice)?);
    }

    if let Some(options) = request
        .get("providerOptions")
        .and_then(|options| options.get("openai"))
        .and_then(Value::as_object)
    {
        if let Some(store) = options.get("store").and_then(Value::as_bool) {
            body.insert("store".into(), Value::Bool(store));
        } else {
            body.insert("store".into(), Value::Bool(false));
        }
        if let Some(tier) = options.get("serviceTier").and_then(Value::as_str) {
            body.insert("service_tier".into(), Value::String(tier.into()));
        }
    } else {
        body.insert("store".into(), Value::Bool(false));
    }

    body.insert("stream".into(), Value::Bool(true));

    let id = model_id(request);
    if id.starts_with("gpt-5") {
        body.insert("include".into(), json!(["reasoning.encrypted_content"]));
        body.insert(
            "reasoning".into(),
            json!({ "effort": "medium", "summary": "auto" }),
        );
    }

    if let Some(value) = generation.get("maxTokens").filter(|v| !v.is_null()) {
        body.insert("max_output_tokens".into(), value.clone());
    }
    if let Some(value) = generation.get("temperature").filter(|v| !v.is_null()) {
        body.insert("temperature".into(), value.clone());
    }
    if let Some(value) = generation.get("topP").filter(|v| !v.is_null()) {
        body.insert("top_p".into(), value.clone());
    }
    Ok(Value::Object(body))
}
