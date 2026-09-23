//! Request compilation, response reduction and the (test-injected) transport.

use serde_json::{json, Map, Value};

use crate::body;
use crate::error::{LlmError, LlmResult};
use crate::route::{take_injected_response, Prepared, Response};
use crate::schema::{
    merge_generation_options, merge_http_options, merge_provider_options, LLMResponse,
};

fn protocol_id(route: &str) -> &'static str {
    match route {
        "openai-chat" => "openai-chat",
        "openai-compatible-chat" => "openai-compatible-chat",
        "openrouter" | "cloudflare-ai-gateway" | "cloudflare-workers-ai" => {
            "openai-compatible-chat"
        }
        "anthropic-messages" => "anthropic-messages",
        "gemini" => "gemini",
        "bedrock-converse" => "bedrock-converse",
        "openai-responses" | "openai-responses-websocket" => "openai-responses",
        _ => "openai-chat",
    }
}

fn owned_roots(protocol: &str) -> &'static [&'static str] {
    match protocol {
        "anthropic-messages" => &["model", "system", "messages", "tools"],
        "gemini" => &["contents", "systemInstruction", "tools"],
        "bedrock-converse" => &["modelId", "system", "messages", "tools", "toolConfig"],
        "openai-responses" => &["model", "input", "tools"],
        _ => &["model", "messages", "tools"],
    }
}

fn resolve_options(mut request: Value) -> Value {
    let model_defaults = request
        .get("model")
        .and_then(|model| model.get("defaults"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    let generation = merge_generation_options(&[
        model_defaults.get("generation").cloned(),
        request.get("generation").cloned(),
    ]);
    let provider_options = merge_provider_options(&[
        model_defaults.get("providerOptions").cloned(),
        request.get("providerOptions").cloned(),
    ]);
    let http = merge_http_options(&[
        model_defaults.get("http").cloned(),
        request.get("http").cloned(),
    ]);

    if let Some(generation) = generation {
        request["generation"] = generation;
    } else {
        request.as_object_mut().map(|map| map.remove("generation"));
    }
    match provider_options {
        Some(options) => request["providerOptions"] = options,
        None => {
            request
                .as_object_mut()
                .map(|map| map.remove("providerOptions"));
        }
    }
    match http {
        Some(http) => request["http"] = http,
        None => {
            request.as_object_mut().map(|map| map.remove("http"));
        }
    }
    request
}

fn fake_body(request: &Value) -> Value {
    let mut lines: Vec<String> = Vec::new();
    for message in request
        .get("messages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        for part in message
            .get("content")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            if part.get("type").and_then(Value::as_str) == Some("text") {
                lines.push(part["text"].as_str().unwrap_or("").to_string());
            }
        }
    }
    for tool in request
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        lines.push(format!(
            "tool:{}:{}",
            tool.get("name").and_then(Value::as_str).unwrap_or(""),
            tool.get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }
    json!({ "body": lines.join("\n") })
}

fn is_fake(route: &str) -> bool {
    route == "fake" || route == "gemini-fake"
}

fn apply_http_overlays(protocol: &str, mut body: Value, http: Option<&Value>) -> LlmResult<Value> {
    let Some(http) = http else {
        return Ok(body);
    };
    if let Some(overlay) = http.get("body") {
        let roots = owned_roots(protocol);
        let present: Vec<&str> = roots
            .iter()
            .copied()
            .filter(|key| overlay.get(*key).is_some())
            .collect();
        if !present.is_empty() {
            return Err(LlmError::InvalidRequest(format!(
                "http.body cannot overlay protocol-owned field(s): {}",
                present.join(", ")
            )));
        }
        body = crate::schema::merge_json_records(&[Some(body), Some(overlay.clone())])
            .unwrap_or(Value::Null);
    }
    if let Some(query) = http.get("query") {
        body["query"] = query.clone();
    }
    if let Some(headers) = http.get("headers") {
        body["headers"] = headers.clone();
    }
    Ok(body)
}

/// Compile a request into its prepared wire body.
pub fn prepare(request: Value) -> LlmResult<Prepared> {
    let route = request
        .get("model")
        .and_then(|model| model.get("route"))
        .and_then(|route| route.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| LlmError::InvalidRequest("request model requires a route id".into()))?
        .to_string();

    let resolved = resolve_options(request);
    let lowered = crate::cache_policy::apply_cache_policy(resolved.clone())?;

    let body = if is_fake(&route) {
        fake_body(&lowered)
    } else {
        let protocol = protocol_id(&route);
        body::build(protocol, &lowered)?
    };
    let protocol = if is_fake(&route) {
        "fake".to_string()
    } else {
        protocol_id(&route).to_string()
    };
    let body = apply_http_overlays(&protocol, body, lowered.get("http"))?;

    let model = lowered.get("model").cloned().unwrap_or(Value::Null);
    Ok(Prepared {
        route,
        protocol,
        model,
        body,
        metadata: Some(json!({ "transport": "http" })),
    })
}

fn canned_events(canned: &Value) -> LlmResult<Vec<Value>> {
    if let Some(call) = canned.get("toolCall") {
        return Ok(tool_call_events(call, "tool-calls"));
    }
    if let Some(calls) = canned.get("toolCalls").and_then(Value::as_array) {
        let mut events = vec![json!({ "type": "step-start", "index": 0 })];
        for call in calls {
            events.extend(tool_call_events(call, "tool-calls"));
        }
        events.push(json!({ "type": "step-finish", "index": 0, "reason": "tool-calls" }));
        events.push(json!({ "type": "finish", "reason": "tool-calls" }));
        return Ok(events);
    }
    if let Some(text) = canned.get("text").and_then(Value::as_str) {
        return Ok(vec![
            json!({ "type": "step-start", "index": 0 }),
            json!({ "type": "text-start", "id": "text-0" }),
            json!({ "type": "text-delta", "id": "text-0", "text": text }),
            json!({ "type": "text-end", "id": "text-0" }),
            json!({ "type": "step-finish", "index": 0, "reason": "stop" }),
            json!({ "type": "finish", "reason": "stop" }),
        ]);
    }
    Err(LlmError::InvalidRequest(
        "unsupported canned response".into(),
    ))
}

fn tool_call_events(call: &Value, reason: &str) -> Vec<Value> {
    let id = call.get("id").cloned().unwrap_or(Value::Null);
    let name = call.get("name").cloned().unwrap_or(Value::Null);
    let input = call.get("input").cloned().unwrap_or_else(|| json!({}));
    let arguments = crate::shared::ProviderShared::encode_json(&input);
    let mut events = vec![
        json!({ "type": "tool-input-start", "id": id, "name": name }),
        json!({ "type": "tool-input-delta", "id": id, "name": name, "text": arguments }),
        json!({ "type": "tool-input-end", "id": id, "name": name }),
    ];
    let mut part = Map::new();
    part.insert("type".into(), Value::String("tool-call".into()));
    part.insert("id".into(), id);
    part.insert("name".into(), name);
    part.insert("input".into(), input);
    if let Some(provider_executed) = call.get("providerExecuted") {
        part.insert("providerExecuted".into(), provider_executed.clone());
    }
    events.push(Value::Object(part));
    events.push(json!({ "type": "step-finish", "index": 0, "reason": reason }));
    events.push(json!({ "type": "finish", "reason": reason }));
    events
}

fn fake_events(route: &str, request: &Value) -> LlmResult<Vec<Value>> {
    if route == "fake" {
        let body = fake_body(request);
        let text = format!("echo:{}", serde_json::to_string(&body).unwrap_or_default());
        return Ok(vec![
            json!({ "type": "text-delta", "id": "text-0", "text": text }),
            json!({ "type": "finish", "reason": "stop" }),
        ]);
    }
    Ok(vec![
        json!({ "type": "text-delta", "id": "text-0", "text": "ok" }),
        json!({ "type": "finish", "reason": "stop" }),
    ])
}

/// Stream normalised events for a request.
pub fn stream(request: Value) -> LlmResult<Vec<Value>> {
    let route = request
        .get("model")
        .and_then(|model| model.get("route"))
        .and_then(|route| route.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let resolved = resolve_options(request);

    if is_fake(&route) {
        return fake_events(&route, &resolved);
    }
    if route == "bedrock-converse" {
        let model = resolved.get("model").cloned().unwrap_or(Value::Null);
        let has_auth = model
            .get("auth")
            .map(|auth| auth.get("kind").and_then(Value::as_str) != Some("none"))
            .unwrap_or(false);
        let has_credentials = model
            .get("endpoint")
            .and_then(|endpoint| endpoint.get("credentials"))
            .is_some()
            || model.get("credentials").is_some();
        if !has_auth && !has_credentials {
            return Err(LlmError::InvalidRequest(
                "Bedrock Converse requires either route bearer auth or AWS credentials".into(),
            ));
        }
    }
    if let Some(canned) = resolved.get("canned") {
        return canned_events(canned);
    }
    if let Some(response) = take_injected_response() {
        return parse_injected(&route, &resolved, response);
    }
    Err(LlmError::NotImplemented(
        "llm transport: no recorded response injected",
    ))
}

fn parse_injected(route: &str, request: &Value, response: Value) -> LlmResult<Vec<Value>> {
    let status = response
        .get("status")
        .and_then(Value::as_u64)
        .unwrap_or(200) as u16;
    let body = response
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if !(200..300).contains(&status) {
        return crate::executor::execute(json!({
            "request": { "method": "POST", "url": request.get("model").and_then(|m| m.get("endpoint")).and_then(|e| e.get("baseURL")).cloned().unwrap_or(Value::Null), "headers": {} },
            "response": { "status": status, "body": body },
        }))
        .map(|_| Vec::new());
    }
    let protocol = protocol_id(route);
    crate::sse::parse(protocol, &body)
}

/// Reduce a normalised event stream into a completed response.
pub fn reduce_response(events: Vec<Value>) -> LlmResult<Response> {
    let state = events
        .iter()
        .cloned()
        .fold(LLMResponse::empty(), LLMResponse::reduce);
    let Some(response) = LLMResponse::complete(&state) else {
        return Err(LlmError::Provider(
            "Provider stream ended without a terminal finish event".into(),
        ));
    };
    let finish_reason = response
        .get("finishReason")
        .and_then(Value::as_str)
        .map(str::to_string);
    let usage = response.get("usage").cloned().unwrap_or(Value::Null);
    let message = response.get("message").cloned().unwrap_or(Value::Null);
    let tool_calls = events
        .iter()
        .filter(|event| event.get("type").and_then(Value::as_str) == Some("tool-call"))
        .cloned()
        .collect();
    Ok(Response {
        text: LLMResponse::text(&events),
        reasoning: LLMResponse::reasoning(&events),
        events,
        message,
        usage,
        finish_reason,
        tool_calls,
    })
}
