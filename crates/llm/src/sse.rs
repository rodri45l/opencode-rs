//! SSE chunk parsing into normalised events.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};
use crate::shared::ProviderShared;
use crate::tool_stream::ToolStream;

/// Extract the `data:` payloads from an SSE body, dropping keep-alives.
pub fn data_payloads(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .filter(|data| !data.is_empty() && *data != "[DONE]")
        .map(str::to_string)
        .collect()
}

/// Parse a provider SSE body into normalised events.
pub fn parse(protocol: &str, body: &str) -> LlmResult<Vec<Value>> {
    match protocol {
        "openai-chat" | "openai-compatible-chat" => parse_openai_chat(body),
        "anthropic-messages" => parse_anthropic(body),
        "bedrock-converse" => parse_bedrock(body),
        "gemini" => parse_gemini(body),
        "openai-responses" => parse_openai_responses(body),
        _ => Err(LlmError::NotImplemented(
            "sse parsing for this protocol is not implemented",
        )),
    }
}

fn map_finish_reason(reason: &str) -> &'static str {
    match reason {
        "stop" => "stop",
        "length" => "length",
        "content_filter" => "content-filter",
        "function_call" | "tool_calls" => "tool-calls",
        _ => "unknown",
    }
}

fn map_usage(usage: &Value) -> Value {
    let prompt = usage.get("prompt_tokens").and_then(Value::as_i64);
    let completion = usage.get("completion_tokens").and_then(Value::as_i64);
    let cached = usage
        .get("prompt_tokens_details")
        .and_then(|details| details.get("cached_tokens"))
        .and_then(Value::as_i64);
    let reasoning = usage
        .get("completion_tokens_details")
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(Value::as_i64);
    let total = usage.get("total_tokens").and_then(Value::as_i64);

    let mut out = Map::new();
    if let Some(value) = prompt {
        out.insert("inputTokens".into(), json!(value));
    }
    if let Some(value) = completion {
        out.insert("outputTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::subtract_tokens(prompt, cached) {
        out.insert("nonCachedInputTokens".into(), json!(value));
    }
    if let Some(value) = cached {
        out.insert("cacheReadInputTokens".into(), json!(value));
    }
    if let Some(value) = reasoning {
        out.insert("reasoningTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::total_tokens(prompt, completion, total) {
        out.insert("totalTokens".into(), json!(value));
    }
    Value::Object(out)
}

struct OpenAiState {
    events: Vec<Value>,
    step_started: bool,
    text_started: bool,
    reasoning_started: bool,
    tools: Value,
    tool_events: Vec<Value>,
    usage: Option<Value>,
    finish_reason: Option<&'static str>,
}

impl OpenAiState {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            step_started: false,
            text_started: false,
            reasoning_started: false,
            tools: ToolStream::empty(),
            tool_events: Vec::new(),
            usage: None,
            finish_reason: None,
        }
    }

    fn ensure_step(&mut self) {
        if !self.step_started {
            self.step_started = true;
            self.events
                .push(json!({ "type": "step-start", "index": 0 }));
        }
    }

    fn close_reasoning(&mut self) {
        if self.reasoning_started {
            self.reasoning_started = false;
            self.events
                .push(json!({ "type": "reasoning-end", "id": "reasoning-0" }));
        }
    }

    fn close_text(&mut self) {
        if self.text_started {
            self.text_started = false;
            self.events
                .push(json!({ "type": "text-end", "id": "text-0" }));
        }
    }
}

fn parse_openai_chat(body: &str) -> LlmResult<Vec<Value>> {
    let mut state = OpenAiState::new();
    for payload in data_payloads(body) {
        let chunk: Value = serde_json::from_str(&payload).map_err(|_| {
            LlmError::InvalidRequest("Invalid openai/openai-chat stream event".into())
        })?;
        if let Some(usage) = chunk.get("usage").filter(|value| !value.is_null()) {
            state.usage = Some(map_usage(usage));
        }
        let choice = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .cloned();
        let Some(choice) = choice else { continue };
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            state.finish_reason = Some(map_finish_reason(reason));
        }
        let delta = choice.get("delta").cloned().unwrap_or_else(|| json!({}));

        if let Some(reasoning) = delta.get("reasoning_content").and_then(Value::as_str) {
            state.ensure_step();
            if !state.reasoning_started {
                state.reasoning_started = true;
                state
                    .events
                    .push(json!({ "type": "reasoning-start", "id": "reasoning-0" }));
            }
            state.events.push(json!({
                "type": "reasoning-delta",
                "id": "reasoning-0",
                "text": reasoning,
            }));
        }
        if let Some(content) = delta.get("content").and_then(Value::as_str) {
            if !content.is_empty() {
                state.ensure_step();
                state.close_reasoning();
                if !state.text_started {
                    state.text_started = true;
                    state
                        .events
                        .push(json!({ "type": "text-start", "id": "text-0" }));
                }
                state.events.push(json!({
                    "type": "text-delta",
                    "id": "text-0",
                    "text": content,
                }));
            }
        }
        if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
            state.ensure_step();
            state.close_reasoning();
            for tool in tool_calls {
                let index = tool.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                let delta = json!({
                    "id": tool.get("id").cloned().unwrap_or(Value::Null),
                    "name": tool.get("function").and_then(|f| f.get("name")).cloned().unwrap_or(Value::Null),
                    "text": tool.get("function").and_then(|f| f.get("arguments")).and_then(Value::as_str).unwrap_or(""),
                });
                let next = ToolStream::append_or_start(
                    state.tools.clone(),
                    index,
                    delta,
                    "OpenAI Chat tool call delta is missing id or name",
                )?;
                if let Some(events) = next.get("events").and_then(Value::as_array) {
                    state.events.extend(events.iter().cloned());
                }
                state.tools = next;
            }
        }

        if state.finish_reason.is_some()
            && state
                .tools
                .get("tools")
                .and_then(Value::as_object)
                .map(|tools| !tools.is_empty())
                .unwrap_or(false)
        {
            let finished = ToolStream::finish_all(state.tools.clone())?;
            if let Some(events) = finished.get("events").and_then(Value::as_array) {
                state.events.extend(events.iter().cloned());
            }
            state.tools = finished;
        }
    }

    state.close_reasoning();
    state.close_text();
    state.events.append(&mut state.tool_events);

    let has_tool_calls = state
        .tools
        .get("tools")
        .and_then(Value::as_object)
        .map(|tools| !tools.is_empty())
        .unwrap_or(false);
    let mut reason = state.finish_reason.unwrap_or("stop");
    if reason == "stop" && has_tool_calls {
        reason = "tool-calls";
    }
    let mut step_finish = Map::new();
    step_finish.insert("type".into(), Value::String("step-finish".into()));
    step_finish.insert("index".into(), json!(0));
    step_finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &state.usage {
        step_finish.insert("usage".into(), usage.clone());
    }
    state.events.push(Value::Object(step_finish));
    let mut finish = Map::new();
    finish.insert("type".into(), Value::String("finish".into()));
    finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &state.usage {
        finish.insert("usage".into(), usage.clone());
    }
    state.events.push(Value::Object(finish));
    Ok(state.events)
}

// ---------------------------------------------------------------------------
// Anthropic Messages
// ---------------------------------------------------------------------------

fn anthropic_events(body: &str) -> Vec<(Option<String>, String)> {
    let mut events = Vec::new();
    let mut current_event: Option<String> = None;
    let mut current_data: Option<String> = None;
    let flush = |event: &mut Option<String>,
                 data: &mut Option<String>,
                 out: &mut Vec<(Option<String>, String)>| {
        if let Some(data) = data.take() {
            out.push((event.take(), data));
        } else {
            *event = None;
        }
    };
    for line in body.lines() {
        if let Some(name) = line.strip_prefix("event:") {
            current_event = Some(name.trim().to_string());
        } else if let Some(data) = line.strip_prefix("data:") {
            current_data = Some(data.trim().to_string());
        } else if line.trim().is_empty() {
            flush(&mut current_event, &mut current_data, &mut events);
        }
    }
    flush(&mut current_event, &mut current_data, &mut events);
    events
}

fn anthropic_usage(usage: &Value) -> Value {
    let input = usage.get("input_tokens").and_then(Value::as_i64);
    let cache_read = usage.get("cache_read_input_tokens").and_then(Value::as_i64);
    let cache_write = usage
        .get("cache_creation_input_tokens")
        .and_then(Value::as_i64);
    let output = usage.get("output_tokens").and_then(Value::as_i64);
    let inclusive_input = ProviderShared::sum_tokens(&[input, cache_read, cache_write]);

    let mut out = Map::new();
    if let Some(value) = inclusive_input {
        out.insert("inputTokens".into(), json!(value));
    }
    if let Some(value) = output {
        out.insert("outputTokens".into(), json!(value));
    }
    if let Some(value) = input {
        out.insert("nonCachedInputTokens".into(), json!(value));
    }
    if let Some(value) = cache_read {
        out.insert("cacheReadInputTokens".into(), json!(value));
    }
    if let Some(value) = cache_write {
        out.insert("cacheWriteInputTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::total_tokens(inclusive_input, output, None) {
        out.insert("totalTokens".into(), json!(value));
    }
    Value::Object(out)
}

fn merge_usage(current: &mut Option<Value>, update: Value) {
    let mut merged = current
        .as_ref()
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(map) = update.as_object() {
        for (key, value) in map {
            merged.insert(key.clone(), value.clone());
        }
    }
    *current = Some(Value::Object(merged));
}

fn anthropic_finish(reason: &str) -> &'static str {
    match reason {
        "end_turn" | "stop_sequence" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool-calls",
        _ => "unknown",
    }
}

#[derive(Default, Clone)]
struct AnthropicBlock {
    kind: String,
    id: Option<String>,
    name: Option<String>,
    input: String,
    provider_executed: bool,
    signature: Option<String>,
    started: bool,
}

fn parse_anthropic(body: &str) -> LlmResult<Vec<Value>> {
    let mut events: Vec<Value> = Vec::new();
    let mut blocks: std::collections::BTreeMap<i64, AnthropicBlock> = Default::default();
    let mut usage: Option<Value> = None;
    let mut finish_reason: Option<&'static str> = None;
    let mut step_started = false;
    let mut provider_error: Option<Value> = None;

    let ensure_step = |events: &mut Vec<Value>, step_started: &mut bool| {
        if !*step_started {
            *step_started = true;
            events.push(json!({ "type": "step-start", "index": 0 }));
        }
    };

    for (_name, data) in anthropic_events(body) {
        let Ok(event) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("message_start") => {
                if let Some(usage_value) = event
                    .get("message")
                    .and_then(|message| message.get("usage"))
                {
                    usage = Some(anthropic_usage(usage_value));
                }
            }
            Some("content_block_start") => {
                let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                let block = event
                    .get("content_block")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let kind = block
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let mut state = AnthropicBlock {
                    kind: kind.clone(),
                    id: block.get("id").and_then(Value::as_str).map(str::to_string),
                    name: block
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    input: String::new(),
                    provider_executed: kind == "server_tool_use",
                    signature: None,
                    started: false,
                };
                if let Some(initial) = block.get("thinking").and_then(Value::as_str) {
                    state.input = initial.to_string();
                }
                blocks.insert(index, state);
            }
            Some("content_block_delta") => {
                let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                let delta = event.get("delta").cloned().unwrap_or_else(|| json!({}));
                let block = blocks.entry(index).or_default();
                match delta.get("type").and_then(Value::as_str) {
                    Some("text_delta") => {
                        ensure_step(&mut events, &mut step_started);
                        let text = delta.get("text").and_then(Value::as_str).unwrap_or("");
                        events.push(json!({ "type": "text-start", "id": "text-0" }));
                        events.push(json!({ "type": "text-delta", "id": "text-0", "text": text }));
                        block.started = true;
                    }
                    Some("thinking_delta") => {
                        ensure_step(&mut events, &mut step_started);
                        let text = delta.get("thinking").and_then(Value::as_str).unwrap_or("");
                        events.push(json!({ "type": "reasoning-start", "id": "reasoning-0" }));
                        events.push(
                            json!({ "type": "reasoning-delta", "id": "reasoning-0", "text": text }),
                        );
                        block.started = true;
                    }
                    Some("signature_delta") => {
                        block.signature = delta
                            .get("signature")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    }
                    Some("input_json_delta") => {
                        let text = delta
                            .get("partial_json")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if !block.started {
                            block.started = true;
                            events.push(json!({
                                "type": "tool-input-start",
                                "id": block.id.clone().map(Value::String).unwrap_or(Value::Null),
                                "name": block.name.clone().map(Value::String).unwrap_or(Value::Null),
                            }));
                        }
                        block.input.push_str(text);
                        events.push(json!({
                            "type": "tool-input-delta",
                            "id": block.id.clone().map(Value::String).unwrap_or(Value::Null),
                            "name": block.name.clone().map(Value::String).unwrap_or(Value::Null),
                            "text": text,
                        }));
                    }
                    _ => {}
                }
            }
            Some("content_block_stop") => {
                let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                let Some(block) = blocks.get(&index).cloned() else {
                    continue;
                };
                match block.kind.as_str() {
                    "text" => {
                        events.push(json!({ "type": "text-end", "id": "text-0" }));
                    }
                    "thinking" => {
                        let mut end = json!({ "type": "reasoning-end", "id": "reasoning-0" });
                        if let Some(signature) = &block.signature {
                            end["providerMetadata"] =
                                json!({ "anthropic": { "signature": signature } });
                        }
                        events.push(end);
                    }
                    "tool_use" | "server_tool_use" => {
                        let input = if block.input.is_empty() {
                            json!({})
                        } else {
                            serde_json::from_str(&block.input).unwrap_or_else(|_| json!({}))
                        };
                        let id = block.id.clone().unwrap_or_default();
                        let name = block.name.clone().unwrap_or_default();
                        events.push(json!({ "type": "tool-input-end", "id": id, "name": name }));
                        let mut call = Map::new();
                        call.insert("type".into(), Value::String("tool-call".into()));
                        call.insert("id".into(), Value::String(id));
                        call.insert("name".into(), Value::String(name));
                        call.insert("input".into(), input);
                        if block.provider_executed {
                            call.insert("providerExecuted".into(), Value::Bool(true));
                        }
                        events.push(Value::Object(call));
                    }
                    _ => {}
                }
            }
            Some("message_delta") => {
                if let Some(reason) = event
                    .get("delta")
                    .and_then(|delta| delta.get("stop_reason"))
                    .and_then(Value::as_str)
                {
                    finish_reason = Some(anthropic_finish(reason));
                }
                if let Some(update) = event.get("usage") {
                    merge_usage(&mut usage, anthropic_usage(update));
                }
            }
            Some("error") => {
                let error = event.get("error").cloned().unwrap_or_else(|| json!({}));
                let kind = error.get("type").and_then(Value::as_str).unwrap_or("error");
                let message = error.get("message").and_then(Value::as_str).unwrap_or("");
                let text = format!("{kind}: {message}");
                let mut rendered = json!({ "type": "provider-error", "message": text });
                if crate::provider_error::is_context_overflow(&text) {
                    rendered["classification"] = Value::String("context-overflow".into());
                }
                provider_error = Some(rendered);
            }
            _ => {}
        }
    }

    if let Some(error) = provider_error {
        return Ok(vec![error]);
    }

    let mut reason = finish_reason.unwrap_or("stop");
    if reason == "stop" && events.iter().any(|event| event["type"] == "tool-call") {
        reason = "tool-calls";
    }
    let mut step_finish = Map::new();
    step_finish.insert("type".into(), Value::String("step-finish".into()));
    step_finish.insert("index".into(), json!(0));
    step_finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        step_finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(step_finish));
    let mut finish = Map::new();
    finish.insert("type".into(), Value::String("finish".into()));
    finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(finish));
    Ok(events)
}

// ---------------------------------------------------------------------------
// Bedrock Converse (decoded event-stream frames)
// ---------------------------------------------------------------------------

fn bedrock_usage(usage: &Value) -> Value {
    let input = usage.get("inputTokens").and_then(Value::as_i64);
    let output = usage.get("outputTokens").and_then(Value::as_i64);
    let total = usage.get("totalTokens").and_then(Value::as_i64);
    let mut out = Map::new();
    if let Some(value) = input {
        out.insert("inputTokens".into(), json!(value));
    }
    if let Some(value) = output {
        out.insert("outputTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::total_tokens(input, output, total) {
        out.insert("totalTokens".into(), json!(value));
    }
    Value::Object(out)
}

fn bedrock_finish(reason: &str) -> &'static str {
    match reason {
        "end_turn" | "stop_sequence" => "stop",
        "max_tokens" => "length",
        "tool_use" => "tool-calls",
        _ => "unknown",
    }
}

fn parse_bedrock(body: &str) -> LlmResult<Vec<Value>> {
    let frames: Value = serde_json::from_str(body)
        .map_err(|_| LlmError::InvalidRequest("Invalid bedrock-converse event frame".into()))?;
    let frames = frames.as_array().cloned().unwrap_or_default();

    let mut events: Vec<Value> = Vec::new();
    let mut tools = ToolStream::empty();
    let mut usage: Option<Value> = None;
    let mut finish_reason: Option<&'static str> = None;
    let mut step_started = false;
    let mut text_started = false;

    for frame in frames {
        let (kind, payload) = if let Some(pair) = frame.as_array() {
            (
                pair.first()
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                pair.get(1).cloned().unwrap_or_else(|| json!({})),
            )
        } else {
            (
                frame
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                frame.clone(),
            )
        };

        let ensure_step = |events: &mut Vec<Value>, step_started: &mut bool| {
            if !*step_started {
                *step_started = true;
                events.push(json!({ "type": "step-start", "index": 0 }));
            }
        };

        match kind.as_str() {
            "messageStart" => ensure_step(&mut events, &mut step_started),
            "contentBlockStart" => {
                ensure_step(&mut events, &mut step_started);
                if let Some(tool) = payload.get("start").and_then(|start| start.get("toolUse")) {
                    let index = payload
                        .get("contentBlockIndex")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    events.push(json!({
                        "type": "tool-input-start",
                        "id": tool.get("toolUseId").cloned().unwrap_or(Value::Null),
                        "name": tool.get("name").cloned().unwrap_or(Value::Null),
                    }));
                    tools = ToolStream::start(
                        tools,
                        json!(index),
                        json!({
                            "id": tool.get("toolUseId").cloned().unwrap_or(Value::Null),
                            "name": tool.get("name").cloned().unwrap_or(Value::Null),
                            "input": "",
                        }),
                    );
                }
            }
            "contentBlockDelta" => {
                ensure_step(&mut events, &mut step_started);
                let delta = payload.get("delta").cloned().unwrap_or_else(|| json!({}));
                if let Some(text) = delta.get("text").and_then(Value::as_str) {
                    if !text_started {
                        text_started = true;
                        events.push(json!({ "type": "text-start", "id": "text-0" }));
                    }
                    events.push(json!({ "type": "text-delta", "id": "text-0", "text": text }));
                }
                if let Some(reasoning) = delta.get("reasoningContent") {
                    if let Some(text) = reasoning.get("text").and_then(Value::as_str) {
                        events.push(json!({ "type": "reasoning-start", "id": "reasoning-0" }));
                        events.push(json!({
                            "type": "reasoning-delta",
                            "id": "reasoning-0",
                            "text": text,
                        }));
                    }
                    if let Some(signature) = reasoning.get("signature").and_then(Value::as_str) {
                        events.push(json!({
                            "type": "reasoning-end",
                            "id": "reasoning-0",
                            "providerMetadata": { "amazon-bedrock": { "signature": signature } },
                        }));
                    }
                }
                if let Some(tool) = delta.get("toolUse") {
                    let index = payload
                        .get("contentBlockIndex")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as usize;
                    let text = tool.get("input").and_then(Value::as_str).unwrap_or("");
                    let next = ToolStream::append_existing(
                        tools.clone(),
                        index,
                        text,
                        "Bedrock tool call delta without start",
                    )?;
                    if let Some(events_array) = next.get("events").and_then(Value::as_array) {
                        events.extend_from_slice(events_array);
                    }
                    tools = next;
                }
            }
            "contentBlockStop" => {
                if text_started {
                    text_started = false;
                    events.push(json!({ "type": "text-end", "id": "text-0" }));
                }
            }
            "messageStop" => {
                if let Some(reason) = payload.get("stopReason").and_then(Value::as_str) {
                    finish_reason = Some(bedrock_finish(reason));
                }
            }
            "metadata" => {
                if let Some(update) = payload.get("usage") {
                    usage = Some(bedrock_usage(update));
                }
            }
            "throttlingException" => {
                events.push(json!({
                    "type": "provider-error",
                    "message": payload.get("message").and_then(Value::as_str).unwrap_or("Throttled"),
                    "retryable": true,
                }));
            }
            "validationException" => {
                let message = payload
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Validation error");
                let mut event = json!({
                    "type": "provider-error",
                    "message": message,
                    "retryable": false,
                });
                if crate::provider_error::is_context_overflow(message) {
                    event["classification"] = Value::String("context-overflow".into());
                }
                events.push(event);
            }
            _ => {}
        }
    }

    if text_started {
        events.push(json!({ "type": "text-end", "id": "text-0" }));
    }
    let has_tools = tools
        .get("tools")
        .and_then(Value::as_object)
        .map(|tools| !tools.is_empty())
        .unwrap_or(false);
    if has_tools {
        let finished = ToolStream::finish_all(tools)?;
        if let Some(events_array) = finished.get("events").and_then(Value::as_array) {
            events.extend_from_slice(events_array);
        }
    }
    let mut reason = finish_reason.unwrap_or("stop");
    if reason == "stop" && events.iter().any(|event| event["type"] == "tool-call") {
        reason = "tool-calls";
    }
    let mut step_finish = Map::new();
    step_finish.insert("type".into(), Value::String("step-finish".into()));
    step_finish.insert("index".into(), json!(0));
    step_finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        step_finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(step_finish));
    let mut finish = Map::new();
    finish.insert("type".into(), Value::String("finish".into()));
    finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(finish));
    Ok(events)
}

// ---------------------------------------------------------------------------
// Gemini
// ---------------------------------------------------------------------------

fn gemini_usage(usage: &Value) -> Value {
    let input = usage.get("promptTokenCount").and_then(Value::as_i64);
    let output = usage.get("candidatesTokenCount").and_then(Value::as_i64);
    let reasoning = usage.get("thoughtsTokenCount").and_then(Value::as_i64);
    let cached = usage.get("cachedContentTokenCount").and_then(Value::as_i64);
    let total = usage.get("totalTokenCount").and_then(Value::as_i64);
    let mut out = Map::new();
    if let Some(value) = input {
        out.insert("inputTokens".into(), json!(value));
    }
    if let Some(value) = output {
        out.insert("outputTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::subtract_tokens(input, cached) {
        out.insert("nonCachedInputTokens".into(), json!(value));
    }
    if let Some(value) = cached {
        out.insert("cacheReadInputTokens".into(), json!(value));
    }
    if let Some(value) = reasoning {
        out.insert("reasoningTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::total_tokens(input, output, total) {
        out.insert("totalTokens".into(), json!(value));
    }
    Value::Object(out)
}

fn gemini_finish(reason: &str) -> &'static str {
    match reason {
        "STOP" => "stop",
        "MAX_TOKENS" => "length",
        "SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" => "content-filter",
        _ => "unknown",
    }
}

fn parse_gemini(body: &str) -> LlmResult<Vec<Value>> {
    let mut events: Vec<Value> = Vec::new();
    let mut usage: Option<Value> = None;
    let mut finish_reason: Option<&'static str> = None;
    let mut step_started = false;
    let mut text_started = false;
    let mut reasoning_started = false;
    let mut tool_index = 0usize;

    for payload in data_payloads(body) {
        let chunk: Value = serde_json::from_str(&payload)
            .map_err(|_| LlmError::InvalidRequest("Invalid gemini stream event".into()))?;
        if let Some(metadata) = chunk.get("usageMetadata") {
            usage = Some(gemini_usage(metadata));
        }
        let Some(candidate) = chunk
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|candidates| candidates.first())
        else {
            continue;
        };
        if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
            finish_reason = Some(gemini_finish(reason));
        }
        let parts = candidate
            .get("content")
            .and_then(|content| content.get("parts"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for part in parts {
            if !step_started {
                step_started = true;
                events.push(json!({ "type": "step-start", "index": 0 }));
            }
            if let Some(call) = part.get("functionCall") {
                if reasoning_started {
                    reasoning_started = false;
                    events.push(json!({ "type": "reasoning-end", "id": "reasoning-0" }));
                }
                let id = format!("tool_{tool_index}");
                tool_index += 1;
                let name = call.get("name").cloned().unwrap_or(Value::Null);
                let input = call.get("args").cloned().unwrap_or_else(|| json!({}));
                events.push(json!({ "type": "tool-input-start", "id": id, "name": name }));
                events.push(json!({
                    "type": "tool-input-delta",
                    "id": id,
                    "name": name,
                    "text": serde_json::to_string(&input).unwrap_or_default(),
                }));
                events.push(json!({ "type": "tool-input-end", "id": id, "name": name }));
                let mut event =
                    json!({ "type": "tool-call", "id": id, "name": name, "input": input });
                if let Some(signature) = part.get("thoughtSignature").and_then(Value::as_str) {
                    event["providerMetadata"] =
                        json!({ "google": { "thoughtSignature": signature } });
                }
                events.push(event);
                continue;
            }
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                let thought = part
                    .get("thought")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if thought {
                    if !reasoning_started {
                        reasoning_started = true;
                        events.push(json!({ "type": "reasoning-start", "id": "reasoning-0" }));
                    }
                    events.push(
                        json!({ "type": "reasoning-delta", "id": "reasoning-0", "text": text }),
                    );
                    if let Some(signature) = part.get("thoughtSignature").and_then(Value::as_str) {
                        reasoning_started = false;
                        events.push(json!({
                            "type": "reasoning-end",
                            "id": "reasoning-0",
                            "providerMetadata": { "google": { "thoughtSignature": signature } },
                        }));
                    }
                } else {
                    if reasoning_started {
                        reasoning_started = false;
                        events.push(json!({ "type": "reasoning-end", "id": "reasoning-0" }));
                    }
                    if !text_started {
                        text_started = true;
                        events.push(json!({ "type": "text-start", "id": "text-0" }));
                    }
                    events.push(json!({ "type": "text-delta", "id": "text-0", "text": text }));
                }
            }
        }
    }

    if reasoning_started {
        events.push(json!({ "type": "reasoning-end", "id": "reasoning-0" }));
    }
    if text_started {
        events.push(json!({ "type": "text-end", "id": "text-0" }));
    }
    let has_tool_calls = events.iter().any(|event| event["type"] == "tool-call");
    let mut reason = finish_reason.unwrap_or("stop");
    if reason == "stop" && has_tool_calls {
        reason = "tool-calls";
    }
    let mut step_finish = Map::new();
    step_finish.insert("type".into(), Value::String("step-finish".into()));
    step_finish.insert("index".into(), json!(0));
    step_finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        step_finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(step_finish));
    let mut finish = Map::new();
    finish.insert("type".into(), Value::String("finish".into()));
    finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(finish));
    Ok(events)
}

// ---------------------------------------------------------------------------
// OpenAI Responses
// ---------------------------------------------------------------------------

fn responses_usage(usage: &Value) -> Value {
    let input = usage.get("input_tokens").and_then(Value::as_i64);
    let output = usage.get("output_tokens").and_then(Value::as_i64);
    let cached = usage
        .get("input_tokens_details")
        .and_then(|details| details.get("cached_tokens"))
        .and_then(Value::as_i64);
    let reasoning = usage
        .get("output_tokens_details")
        .and_then(|details| details.get("reasoning_tokens"))
        .and_then(Value::as_i64);
    let total = usage.get("total_tokens").and_then(Value::as_i64);
    let mut out = Map::new();
    if let Some(value) = input {
        out.insert("inputTokens".into(), json!(value));
    }
    if let Some(value) = output {
        out.insert("outputTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::subtract_tokens(input, cached) {
        out.insert("nonCachedInputTokens".into(), json!(value));
    }
    if let Some(value) = cached {
        out.insert("cacheReadInputTokens".into(), json!(value));
    }
    if let Some(value) = reasoning {
        out.insert("reasoningTokens".into(), json!(value));
    }
    if let Some(value) = ProviderShared::total_tokens(input, output, total) {
        out.insert("totalTokens".into(), json!(value));
    }
    Value::Object(out)
}

fn parse_openai_responses(body: &str) -> LlmResult<Vec<Value>> {
    let mut events: Vec<Value> = Vec::new();
    let mut usage: Option<Value> = None;
    let mut step_started = false;
    let mut text_id: Option<String> = None;
    let mut reasoning_id: Option<String> = None;
    let mut tools: std::collections::BTreeMap<String, Value> = Default::default();
    let mut provider_error: Option<Value> = None;

    let ensure_step = |events: &mut Vec<Value>, step_started: &mut bool| {
        if !*step_started {
            *step_started = true;
            events.push(json!({ "type": "step-start", "index": 0 }));
        }
    };

    for payload in data_payloads(body) {
        let Ok(event) = serde_json::from_str::<Value>(&payload) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("response.output_text.delta") => {
                ensure_step(&mut events, &mut step_started);
                let id = event
                    .get("item_id")
                    .and_then(Value::as_str)
                    .unwrap_or("text-0")
                    .to_string();
                if text_id.as_deref() != Some(id.as_str()) {
                    text_id = Some(id.clone());
                    events.push(json!({ "type": "text-start", "id": id }));
                }
                events.push(json!({
                    "type": "text-delta",
                    "id": id,
                    "text": event.get("delta").and_then(Value::as_str).unwrap_or(""),
                }));
            }
            Some("response.reasoning_summary_text.delta") => {
                ensure_step(&mut events, &mut step_started);
                let id = event
                    .get("item_id")
                    .and_then(Value::as_str)
                    .unwrap_or("reasoning-0")
                    .to_string();
                if reasoning_id.as_deref() != Some(id.as_str()) {
                    reasoning_id = Some(id.clone());
                    events.push(json!({ "type": "reasoning-start", "id": id }));
                }
                events.push(json!({
                    "type": "reasoning-delta",
                    "id": id,
                    "text": event.get("delta").and_then(Value::as_str).unwrap_or(""),
                }));
            }
            Some("response.output_item.added") => {
                let item = event.get("item").cloned().unwrap_or_else(|| json!({}));
                if item.get("type").and_then(Value::as_str) == Some("function_call") {
                    ensure_step(&mut events, &mut step_started);
                    let item_id = item
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let call_id = item
                        .get("call_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let name = item
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    events.push(json!({
                        "type": "tool-input-start",
                        "id": call_id,
                        "name": name,
                        "providerMetadata": { "openai": { "itemId": item_id } },
                    }));
                    tools.insert(
                        item_id.clone(),
                        json!({ "call_id": call_id, "name": name, "input": item.get("arguments").and_then(Value::as_str).unwrap_or("") }),
                    );
                }
            }
            Some("response.function_call_arguments.delta") => {
                let item_id = event
                    .get("item_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let delta = event.get("delta").and_then(Value::as_str).unwrap_or("");
                if let Some(entry) = tools.get_mut(&item_id) {
                    let call_id = entry
                        .get("call_id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let name = entry
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let prev = entry
                        .get("input")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    entry["input"] = Value::String(format!("{prev}{delta}"));
                    events.push(json!({
                        "type": "tool-input-delta",
                        "id": call_id,
                        "name": name,
                        "text": delta,
                    }));
                }
            }
            Some("response.output_item.done") => {
                let item = event.get("item").cloned().unwrap_or_else(|| json!({}));
                let item_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                match item.get("type").and_then(Value::as_str) {
                    Some("function_call") => {
                        let call_id = item
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let name = item
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let raw = item.get("arguments").and_then(Value::as_str).unwrap_or("");
                        let input: Value =
                            serde_json::from_str(if raw.is_empty() { "{}" } else { raw })
                                .unwrap_or_else(|_| json!({}));
                        events.push(json!({ "type": "tool-input-end", "id": call_id, "name": name, "providerMetadata": { "openai": { "itemId": item_id } } }));
                        events.push(json!({
                            "type": "tool-call",
                            "id": call_id,
                            "name": name,
                            "input": input,
                            "providerMetadata": { "openai": { "itemId": item_id } },
                        }));
                        tools.remove(&item_id);
                    }
                    Some("web_search_call") => {
                        ensure_step(&mut events, &mut step_started);
                        let id = item
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let action = item.get("action").cloned().unwrap_or_else(|| json!({}));
                        events.push(json!({
                            "type": "tool-call",
                            "id": id,
                            "name": "web_search",
                            "input": action,
                            "providerExecuted": true,
                            "providerMetadata": { "openai": { "itemId": id } },
                        }));
                        events.push(json!({
                            "type": "tool-result",
                            "id": id,
                            "name": "web_search",
                            "result": { "type": "json", "value": item },
                            "providerExecuted": true,
                            "providerMetadata": { "openai": { "itemId": id } },
                        }));
                    }
                    _ => {}
                }
            }
            Some("response.completed") => {
                if let Some(value) = event
                    .get("response")
                    .and_then(|response| response.get("usage"))
                {
                    usage = Some(responses_usage(value));
                }
            }
            Some("response.failed") => {
                let error = event
                    .get("response")
                    .and_then(|response| response.get("error"))
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let code = error.get("code").and_then(Value::as_str).unwrap_or("error");
                let message = error.get("message").and_then(Value::as_str).unwrap_or("");
                provider_error = Some(json!({
                    "type": "provider-error",
                    "message": format!("{code}: {message}"),
                }));
            }
            Some("error") => {
                let code = event.get("code").and_then(Value::as_str).unwrap_or("error");
                let message = event.get("message").and_then(Value::as_str).unwrap_or("");
                provider_error = Some(json!({
                    "type": "provider-error",
                    "message": format!("{code}: {message}"),
                }));
            }
            _ => {}
        }
    }

    if let Some(error) = provider_error {
        return Ok(vec![error]);
    }

    if let Some(id) = &reasoning_id {
        events.push(json!({ "type": "reasoning-end", "id": id }));
    }
    if let Some(id) = &text_id {
        events.push(json!({ "type": "text-end", "id": id }));
    }
    let has_tool_calls = events.iter().any(|event| event["type"] == "tool-call");
    let reason = if has_tool_calls { "tool-calls" } else { "stop" };
    let mut step_finish = Map::new();
    step_finish.insert("type".into(), Value::String("step-finish".into()));
    step_finish.insert("index".into(), json!(0));
    step_finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        step_finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(step_finish));
    let mut finish = Map::new();
    finish.insert("type".into(), Value::String("finish".into()));
    finish.insert("reason".into(), Value::String(reason.into()));
    if let Some(usage) = &usage {
        finish.insert("usage".into(), usage.clone());
    }
    events.push(Value::Object(finish));
    Ok(events)
}
