//! Canonical request/event data used by the ported tests.
//!
//! The Rust port models wire payloads as [`serde_json::Value`]; the helpers here
//! are the ergonomic constructors the reference exposes as schema classes.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};

/// Token accounting for one completion turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usage {
    value: Value,
}

impl Usage {
    /// Wrap a raw usage payload.
    pub fn new(input: Value) -> Self {
        Self { value: input }
    }

    fn int(&self, key: &str) -> Option<i64> {
        self.value.get(key).and_then(Value::as_i64)
    }

    /// Output tokens that were not reasoning tokens, clamped at zero.
    pub fn visible_output_tokens(&self) -> i64 {
        let output = self.int("outputTokens").unwrap_or(0);
        let reasoning = self.int("reasoningTokens").unwrap_or(0);
        (output - reasoning).max(0)
    }

    /// Total tokens when the provider reported them.
    pub fn total_tokens(&self) -> Option<i64> {
        self.int("totalTokens")
    }

    /// Underlying payload.
    pub fn as_value(&self) -> &Value {
        &self.value
    }
}

fn is_record(value: &Value) -> bool {
    value.is_object()
}

/// Deep-merge JSON records; later values replace arrays, primitives and nulls.
pub fn merge_json_records(items: &[Option<Value>]) -> Option<Value> {
    let defined: Vec<&Value> = items.iter().filter_map(|item| item.as_ref()).collect();
    if defined.is_empty() {
        return None;
    }
    let mut result = Map::new();
    for item in defined {
        let Some(map) = item.as_object() else {
            continue;
        };
        for (key, value) in map {
            if value.is_null() {
                result.insert(key.clone(), Value::Null);
                continue;
            }
            if value.is_object() && result.get(key).map(is_record).unwrap_or(false) {
                let merged = merge_json_records(&[result.get(key).cloned(), Some(value.clone())]);
                if let Some(merged) = merged {
                    result.insert(key.clone(), merged);
                }
            } else {
                result.insert(key.clone(), value.clone());
            }
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(Value::Object(result))
    }
}

/// Merge provider-option layers (`{ provider: { ... } }`).
pub fn merge_provider_options(items: &[Option<Value>]) -> Option<Value> {
    let mut result = Map::new();
    for item in items.iter().flatten() {
        let Some(map) = item.as_object() else {
            continue;
        };
        for (provider, options) in map {
            let merged =
                merge_json_records(&[result.get(provider).cloned(), Some(options.clone())]);
            if let Some(merged) = merged {
                result.insert(provider.clone(), merged);
            }
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(Value::Object(result))
    }
}

fn merge_string_records(items: &[Option<Value>]) -> Option<Value> {
    let mut result = Map::new();
    let mut any = false;
    for item in items.iter().flatten() {
        let Some(map) = item.as_object() else {
            continue;
        };
        for (key, value) in map {
            if !value.is_null() {
                result.insert(key.clone(), value.clone());
                any = true;
            }
        }
    }
    if any {
        Some(Value::Object(result))
    } else {
        None
    }
}

/// Merge HTTP option layers (`body`, `headers`, `query`).
pub fn merge_http_options(items: &[Option<Value>]) -> Option<Value> {
    let body = merge_json_records(
        &items
            .iter()
            .map(|item| item.as_ref().and_then(|v| v.get("body").cloned()))
            .collect::<Vec<_>>(),
    );
    let headers = merge_string_records(
        &items
            .iter()
            .map(|item| item.as_ref().and_then(|v| v.get("headers").cloned()))
            .collect::<Vec<_>>(),
    );
    let query = merge_string_records(
        &items
            .iter()
            .map(|item| item.as_ref().and_then(|v| v.get("query").cloned()))
            .collect::<Vec<_>>(),
    );
    if body.is_none() && headers.is_none() && query.is_none() {
        return None;
    }
    let mut map = Map::new();
    if let Some(body) = body {
        map.insert("body".into(), body);
    }
    if let Some(headers) = headers {
        map.insert("headers".into(), headers);
    }
    if let Some(query) = query {
        map.insert("query".into(), query);
    }
    Some(Value::Object(map))
}

const GENERATION_KEYS: [&str; 8] = [
    "maxTokens",
    "temperature",
    "topP",
    "topK",
    "frequencyPenalty",
    "presencePenalty",
    "seed",
    "stop",
];

/// Merge generation-option layers; later non-null values win per field.
pub fn merge_generation_options(items: &[Option<Value>]) -> Option<Value> {
    let mut map = Map::new();
    for key in GENERATION_KEYS {
        let latest = items
            .iter()
            .rev()
            .flatten()
            .find_map(|item| item.get(key).filter(|v| !v.is_null()).cloned());
        if let Some(value) = latest {
            map.insert(key.to_string(), value);
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

/// Request construction helpers.
pub struct LLMRequest;

impl LLMRequest {
    /// Apply a patch, returning the updated request.
    pub fn update(base: Value, patch: Value) -> LlmResult<Value> {
        let mut merged = match base {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        if let Value::Object(patch) = patch {
            for (key, value) in patch {
                merged.insert(key, value);
            }
        }
        Ok(Value::Object(merged))
    }

    /// Extract the input fields from a canonical request.
    pub fn input(request: Value) -> Value {
        request
    }
}

/// Model construction helpers.
pub struct Model;

impl Model {
    /// Build a model descriptor.
    pub fn make(input: Value) -> Value {
        input
    }

    /// Apply a patch, returning the updated model.
    pub fn update(base: Value, patch: Value) -> LlmResult<Value> {
        let mut merged = match base {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        if let Value::Object(patch) = patch {
            for (key, value) in patch {
                merged.insert(key, value);
            }
        }
        Ok(Value::Object(merged))
    }

    /// Extract the input fields from a canonical model.
    pub fn input(model: Value) -> Value {
        model
    }
}

/// Message construction helpers.
pub struct Message;

fn text_part(text: &str) -> Value {
    json!({ "type": "text", "text": text })
}

fn content_array(content: Value) -> Value {
    match content {
        Value::String(text) => Value::Array(vec![text_part(&text)]),
        Value::Array(items) => Value::Array(items),
        other => Value::Array(vec![other]),
    }
}

impl Message {
    /// A user message.
    pub fn user(content: impl Into<Value>) -> Value {
        json!({ "role": "user", "content": content_array(content.into()) })
    }

    /// An assistant message.
    pub fn assistant(content: impl Into<Value>) -> Value {
        json!({ "role": "assistant", "content": content_array(content.into()) })
    }

    /// A system message.
    pub fn system(content: impl Into<Value>) -> Value {
        json!({ "role": "system", "content": content_array(content.into()) })
    }

    /// A tool-result message.
    pub fn tool(content: impl Into<Value>) -> Value {
        let input = content.into();
        let result_type = input
            .get("resultType")
            .and_then(Value::as_str)
            .unwrap_or("json")
            .to_string();
        let raw = input.get("result").cloned().unwrap_or(Value::Null);
        let result = if raw.get("type").is_some() && raw.get("value").is_some() {
            raw
        } else {
            json!({ "type": result_type, "value": raw })
        };
        let mut part = Map::new();
        part.insert("type".into(), Value::String("tool-result".into()));
        if let Some(id) = input.get("id") {
            part.insert("id".into(), id.clone());
        }
        if let Some(name) = input.get("name") {
            part.insert("name".into(), name.clone());
        }
        part.insert("result".into(), result);
        if let Some(provider_executed) = input.get("providerExecuted") {
            part.insert("providerExecuted".into(), provider_executed.clone());
        }
        if let Some(cache) = input.get("cache") {
            part.insert("cache".into(), cache.clone());
        }
        json!({ "role": "tool", "content": [Value::Object(part)] })
    }

    /// Build a message from a raw object.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Event construction helpers.
pub struct LLMEvent;

impl LLMEvent {
    /// A terminal `finish` event.
    pub fn finish(reason: &str, usage: Option<Value>) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("finish".into()));
        map.insert("reason".into(), Value::String(reason.into()));
        if let Some(usage) = usage {
            map.insert("usage".into(), usage);
        }
        Value::Object(map)
    }

    /// A per-step `step-finish` event.
    pub fn step_finish(index: u64, reason: &str, usage: Option<Value>) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("step-finish".into()));
        map.insert("index".into(), json!(index));
        map.insert("reason".into(), Value::String(reason.into()));
        if let Some(usage) = usage {
            map.insert("usage".into(), usage);
        }
        Value::Object(map)
    }

    /// A completed tool call.
    pub fn tool_call(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-call".into()));
        }
        value
    }

    /// A tool result.
    pub fn tool_result(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-result".into()));
        }
        value
    }

    /// A streamed text delta.
    pub fn text_delta(id: &str, text: &str) -> Value {
        json!({ "type": "text-delta", "id": id, "text": text })
    }

    fn is(event: &Value, kind: &str) -> bool {
        event.get("type").and_then(Value::as_str) == Some(kind)
    }

    /// Whether the event is a terminal `finish`.
    pub fn is_finish(event: &Value) -> bool {
        Self::is(event, "finish")
    }

    /// Whether the event is a `tool-call`.
    pub fn is_tool_call(event: &Value) -> bool {
        Self::is(event, "tool-call")
    }

    /// Whether the event is a `tool-result`.
    pub fn is_tool_result(event: &Value) -> bool {
        Self::is(event, "tool-result")
    }

    /// Whether the event is a `tool-error`.
    pub fn is_tool_error(event: &Value) -> bool {
        Self::is(event, "tool-error")
    }

    /// Whether the event is a `step-start`.
    pub fn is_step_start(event: &Value) -> bool {
        Self::is(event, "step-start")
    }

    /// Whether the event is a `step-finish`.
    pub fn is_step_finish(event: &Value) -> bool {
        Self::is(event, "step-finish")
    }
}

/// Content-part guards.
pub struct ContentPart;

impl ContentPart {
    /// Whether the part is a text part.
    pub fn guards_text(part: &Value) -> bool {
        part.get("type").and_then(Value::as_str) == Some("text")
    }

    /// Whether the part is a media part.
    pub fn guards_media(part: &Value) -> bool {
        part.get("type").and_then(Value::as_str) == Some("media")
    }
}

/// Tool choice helpers.
pub struct ToolChoice;

impl ToolChoice {
    /// Build a tool choice from a name, reserved mode, or tool definition.
    pub fn make(input: Value) -> Value {
        match input {
            Value::String(name) => match name.as_str() {
                "auto" | "none" | "required" => json!({ "type": name }),
                _ => json!({ "type": "tool", "name": name }),
            },
            Value::Object(ref map) => {
                if let Some(name) = map.get("name") {
                    json!({ "type": "tool", "name": name })
                } else {
                    input
                }
            }
            other => other,
        }
    }

    /// Build a named tool choice.
    pub fn named(name: &str) -> Value {
        json!({ "type": "tool", "name": name })
    }
}

/// Tool definition helper.
pub struct ToolDefinition;

impl ToolDefinition {
    /// Build a tool definition.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Tool call part helper.
pub struct ToolCallPart;

impl ToolCallPart {
    /// Build a tool call part.
    pub fn make(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-call".into()));
        }
        value
    }
}

/// Tool result part helper.
pub struct ToolResultPart;

impl ToolResultPart {
    /// Build a tool result part.
    pub fn make(input: Value) -> Value {
        let mut value = input;
        if let Value::Object(ref mut map) = value {
            map.insert("type".into(), Value::String("tool-result".into()));
        }
        value
    }
}

/// A manual cache hint attached to a request part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheHint {
    value: Value,
}

impl CacheHint {
    /// Build a cache hint.
    pub fn new(input: Value) -> Self {
        Self { value: input }
    }

    /// Underlying hint payload.
    pub fn as_value(&self) -> &Value {
        &self.value
    }
}

impl serde::Serialize for CacheHint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

fn as_obj(value: &Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    }
}

fn event_usage(event: &Value) -> Option<Value> {
    event.get("usage").filter(|v| !v.is_null()).cloned()
}

/// Response reducer over a normalised event stream.
pub struct LLMResponse;

impl LLMResponse {
    /// Empty reducer state.
    pub fn empty() -> Value {
        json!({
            "events": [],
            "message": { "role": "assistant", "content": [] },
            "textParts": {},
            "reasoningParts": {},
            "toolInputs": {},
        })
    }

    /// Fold one event into the state.
    pub fn reduce(state: Value, event: Value) -> Value {
        let mut state = state;
        let kind = event.get("type").and_then(Value::as_str).unwrap_or("");

        // Append the event and update usage/finish metadata.
        {
            let events = state
                .get_mut("events")
                .and_then(Value::as_array_mut)
                .expect("events array");
            events.push(event.clone());
        }
        if kind == "finish" {
            if let Some(usage) = event_usage(&event) {
                state["usage"] = usage;
            }
            if let Some(reason) = event.get("reason") {
                state["finishReason"] = reason.clone();
            }
        } else if kind == "provider-error" {
            if state.get("finishReason").is_none() {
                state["finishReason"] = Value::String("error".into());
            }
        } else if let Some(usage) = event_usage(&event) {
            state["usage"] = usage;
        }

        match kind {
            "text-start" => ensure_text(&mut state, &event, false),
            "text-delta" => reduce_text_delta(&mut state, &event),
            "text-end" => reduce_text_end(&mut state, &event),
            "reasoning-start" => ensure_text(&mut state, &event, true),
            "reasoning-delta" => reduce_reasoning_delta(&mut state, &event),
            "reasoning-end" => reduce_reasoning_end(&mut state, &event),
            "tool-input-start" => reduce_tool_input_start(&mut state, &event),
            "tool-input-delta" => reduce_tool_input_delta(&mut state, &event),
            "tool-input-end" => reduce_tool_input_end(&mut state, &event),
            "tool-call" => reduce_tool_call(&mut state, &event),
            "tool-result" => append_content(&mut state, tool_result_content(&event)),
            _ => {}
        }
        state
    }

    /// Reduce a full event stream into a completed response, if terminal.
    pub fn from_events(events: Vec<Value>) -> LlmResult<Option<Value>> {
        let state = events.into_iter().fold(Self::empty(), Self::reduce);
        Ok(Self::complete(&state))
    }

    /// Complete a reducer state, if it saw a terminal finish.
    pub fn complete(state: &Value) -> Option<Value> {
        let reason = state.get("finishReason")?;
        let mut response = Map::new();
        response.insert("message".into(), state["message"].clone());
        response.insert("events".into(), state["events"].clone());
        if let Some(usage) = state.get("usage") {
            response.insert("usage".into(), usage.clone());
        }
        response.insert("finishReason".into(), reason.clone());
        Some(Value::Object(response))
    }

    /// Extract concatenated text from an event stream.
    pub fn text(events: &[Value]) -> String {
        events
            .iter()
            .filter(|event| event.get("type").and_then(Value::as_str) == Some("text-delta"))
            .filter_map(|event| event.get("text").and_then(Value::as_str))
            .collect()
    }

    /// Extract concatenated reasoning text from an event stream.
    pub fn reasoning(events: &[Value]) -> String {
        events
            .iter()
            .filter(|event| event.get("type").and_then(Value::as_str) == Some("reasoning-delta"))
            .filter_map(|event| event.get("text").and_then(Value::as_str))
            .collect()
    }
}

fn content_mut(state: &mut Value) -> &mut Vec<Value> {
    state["message"]["content"]
        .as_array_mut()
        .expect("message content")
}

fn append_content(state: &mut Value, part: Value) {
    content_mut(state).push(part);
}

fn replace_content(state: &mut Value, index: usize, part: Value) {
    if let Some(content) = state["message"]["content"].as_array_mut() {
        if index < content.len() {
            content[index] = part;
        }
    }
}

fn text_content(text: &str, metadata: Option<&Value>) -> Value {
    let mut part = Map::new();
    part.insert("type".into(), Value::String("text".into()));
    part.insert("text".into(), Value::String(text.into()));
    if let Some(metadata) = metadata {
        part.insert("providerMetadata".into(), metadata.clone());
    }
    Value::Object(part)
}

fn reasoning_content(text: &str, metadata: Option<&Value>) -> Value {
    let mut part = Map::new();
    part.insert("type".into(), Value::String("reasoning".into()));
    part.insert("text".into(), Value::String(text.into()));
    if let Some(metadata) = metadata {
        part.insert("providerMetadata".into(), metadata.clone());
    }
    Value::Object(part)
}

fn ensure_text(state: &mut Value, event: &Value, reasoning: bool) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let bucket = if reasoning {
        "reasoningParts"
    } else {
        "textParts"
    };
    if state[bucket].get(&id).is_some() {
        return;
    }
    let metadata = event.get("providerMetadata").cloned();
    let index = content_mut(state).len();
    append_content(
        state,
        if reasoning {
            reasoning_content("", metadata.as_ref())
        } else {
            text_content("", metadata.as_ref())
        },
    );
    let mut entry = Map::new();
    entry.insert("contentIndex".into(), json!(index));
    entry.insert("text".into(), Value::String(String::new()));
    if let Some(metadata) = metadata {
        entry.insert("providerMetadata".into(), metadata);
    }
    state[bucket][&id] = Value::Object(entry);
}

fn reduce_text_delta(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if state["textParts"].get(&id).is_none() {
        ensure_text(state, event, false);
    }
    let index = state["textParts"][&id]["contentIndex"]
        .as_u64()
        .unwrap_or(0) as usize;
    let prev = state["textParts"][&id]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let text = format!(
        "{prev}{}",
        event.get("text").and_then(Value::as_str).unwrap_or("")
    );
    let metadata = event
        .get("providerMetadata")
        .cloned()
        .or_else(|| state["textParts"][&id].get("providerMetadata").cloned());
    state["textParts"][&id]["text"] = Value::String(text.clone());
    if let Some(metadata) = metadata.clone() {
        state["textParts"][&id]["providerMetadata"] = metadata;
    }
    replace_content(state, index, text_content(&text, metadata.as_ref()));
}

fn reduce_text_end(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let Some(entry) = state["textParts"].get(&id).cloned() else {
        return;
    };
    let index = entry["contentIndex"].as_u64().unwrap_or(0) as usize;
    let text = entry["text"].as_str().unwrap_or("").to_string();
    let metadata = event
        .get("providerMetadata")
        .cloned()
        .or_else(|| entry.get("providerMetadata").cloned());
    if let Some(metadata) = metadata.clone() {
        state["textParts"][&id]["providerMetadata"] = metadata;
    }
    replace_content(state, index, text_content(&text, metadata.as_ref()));
}

fn reduce_reasoning_delta(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if state["reasoningParts"].get(&id).is_none() {
        ensure_text(state, event, true);
    }
    let index = state["reasoningParts"][&id]["contentIndex"]
        .as_u64()
        .unwrap_or(0) as usize;
    let prev = state["reasoningParts"][&id]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let text = format!(
        "{prev}{}",
        event.get("text").and_then(Value::as_str).unwrap_or("")
    );
    let metadata = event.get("providerMetadata").cloned().or_else(|| {
        state["reasoningParts"][&id]
            .get("providerMetadata")
            .cloned()
    });
    state["reasoningParts"][&id]["text"] = Value::String(text.clone());
    if let Some(metadata) = metadata.clone() {
        state["reasoningParts"][&id]["providerMetadata"] = metadata;
    }
    replace_content(state, index, reasoning_content(&text, metadata.as_ref()));
}

fn reduce_reasoning_end(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let Some(entry) = state["reasoningParts"].get(&id).cloned() else {
        return;
    };
    let index = entry["contentIndex"].as_u64().unwrap_or(0) as usize;
    let text = entry["text"].as_str().unwrap_or("").to_string();
    let metadata = event
        .get("providerMetadata")
        .cloned()
        .or_else(|| entry.get("providerMetadata").cloned());
    if let Some(metadata) = metadata.clone() {
        state["reasoningParts"][&id]["providerMetadata"] = metadata;
    }
    replace_content(state, index, reasoning_content(&text, metadata.as_ref()));
}

fn reduce_tool_input_start(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut entry = Map::new();
    entry.insert(
        "name".into(),
        event
            .get("name")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    entry.insert("text".into(), Value::String(String::new()));
    if let Some(metadata) = event.get("providerMetadata") {
        entry.insert("providerMetadata".into(), metadata.clone());
    }
    state["toolInputs"][&id] = Value::Object(entry);
}

fn reduce_tool_input_delta(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let current = state["toolInputs"].get(&id).cloned().unwrap_or_else(
        || json!({ "name": event.get("name").cloned().unwrap_or(Value::Null), "text": "" }),
    );
    let prev = current["text"].as_str().unwrap_or("").to_string();
    let text = format!(
        "{prev}{}",
        event.get("text").and_then(Value::as_str).unwrap_or("")
    );
    state["toolInputs"][&id] = current;
    state["toolInputs"][&id]["text"] = Value::String(text);
}

fn reduce_tool_input_end(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let current = state["toolInputs"].get(&id).cloned().unwrap_or_else(
        || json!({ "name": event.get("name").cloned().unwrap_or(Value::Null), "text": "" }),
    );
    state["toolInputs"][&id] = current;
    if let Some(name) = event.get("name") {
        state["toolInputs"][&id]["name"] = name.clone();
    }
    if let Some(metadata) = event.get("providerMetadata") {
        state["toolInputs"][&id]["providerMetadata"] = metadata.clone();
    }
}

fn reduce_tool_call(state: &mut Value, event: &Value) {
    let id = event
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if let Some(inputs) = state.get_mut("toolInputs").and_then(Value::as_object_mut) {
        inputs.remove(&id);
    }
    let mut part = Map::new();
    part.insert("type".into(), Value::String("tool-call".into()));
    part.insert("id".into(), event.get("id").cloned().unwrap_or(Value::Null));
    part.insert(
        "name".into(),
        event.get("name").cloned().unwrap_or(Value::Null),
    );
    part.insert(
        "input".into(),
        event.get("input").cloned().unwrap_or(Value::Null),
    );
    if let Some(provider_executed) = event.get("providerExecuted") {
        part.insert("providerExecuted".into(), provider_executed.clone());
    }
    if let Some(metadata) = event.get("providerMetadata") {
        part.insert("providerMetadata".into(), metadata.clone());
    }
    append_content(state, Value::Object(part));
}

fn tool_result_content(event: &Value) -> Value {
    let mut part = Map::new();
    part.insert("type".into(), Value::String("tool-result".into()));
    part.insert("id".into(), event.get("id").cloned().unwrap_or(Value::Null));
    part.insert(
        "name".into(),
        event.get("name").cloned().unwrap_or(Value::Null),
    );
    part.insert(
        "result".into(),
        event.get("result").cloned().unwrap_or(Value::Null),
    );
    if let Some(provider_executed) = event.get("providerExecuted") {
        part.insert("providerExecuted".into(), provider_executed.clone());
    }
    if let Some(metadata) = event.get("providerMetadata") {
        part.insert("providerMetadata".into(), metadata.clone());
    }
    Value::Object(part)
}

/// Build a canonical request from ergonomic input.
pub fn build_request(input: Value) -> LlmResult<Value> {
    let obj = as_obj(&input);
    let mut out = Map::new();

    if let Some(id) = obj.get("id") {
        out.insert("id".into(), id.clone());
    }
    out.insert(
        "model".into(),
        obj.get("model").cloned().unwrap_or(Value::Null),
    );

    let system = match obj.get("system") {
        None | Some(Value::Null) => Value::Array(vec![]),
        Some(Value::String(text)) => Value::Array(vec![text_part(text)]),
        Some(Value::Array(items)) => Value::Array(items.clone()),
        Some(other) => Value::Array(vec![other.clone()]),
    };
    out.insert("system".into(), system);

    let mut messages: Vec<Value> = obj
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(prompt) = obj.get("prompt") {
        messages.push(Message::user(prompt.clone()));
    }
    out.insert("messages".into(), Value::Array(messages));

    out.insert(
        "tools".into(),
        obj.get("tools")
            .cloned()
            .unwrap_or_else(|| Value::Array(vec![])),
    );
    if let Some(tool_choice) = obj.get("toolChoice") {
        out.insert("toolChoice".into(), ToolChoice::make(tool_choice.clone()));
    }
    if let Some(generation) = obj.get("generation") {
        out.insert("generation".into(), generation.clone());
    }
    if let Some(provider_options) = obj.get("providerOptions") {
        out.insert("providerOptions".into(), provider_options.clone());
    }
    if let Some(http) = obj.get("http") {
        out.insert("http".into(), http.clone());
    }
    if let Some(response_format) = obj.get("responseFormat") {
        out.insert("responseFormat".into(), response_format.clone());
    }
    if let Some(cache) = obj.get("cache") {
        out.insert("cache".into(), cache.clone());
    }
    if let Some(metadata) = obj.get("metadata") {
        out.insert("metadata".into(), metadata.clone());
    }
    if let Some(env) = obj.get("env") {
        out.insert("env".into(), env.clone());
    }
    if let Some(canned) = obj.get("canned") {
        out.insert("canned".into(), canned.clone());
    }
    if let Some(schema) = obj.get("schema") {
        out.insert("schema".into(), schema.clone());
    }
    if let Some(json_schema) = obj.get("jsonSchema") {
        out.insert("jsonSchema".into(), json_schema.clone());
    }
    Ok(Value::Object(out))
}

/// Re-exported error alias for callers that only need the type.
pub type SchemaError = LlmError;
