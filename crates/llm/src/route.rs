//! Route, protocol, endpoint, auth and client surfaces.

use std::cell::RefCell;

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};
use crate::schema::merge_json_records;
use crate::Json;

/// HTTP endpoint authoring.
pub struct Endpoint;

impl Endpoint {
    /// Build an endpoint path definition.
    pub fn path(path: &str, options: Value) -> Value {
        let mut value = options;
        if let Value::Object(ref mut map) = value {
            map.insert("path".into(), Value::String(path.into()));
        }
        value
    }

    /// Render the final request URL.
    pub fn render(definition: Value, _context: Value) -> LlmResult<String> {
        let base = definition
            .get("baseURL")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim_end_matches('/')
            .to_string();
        let path = definition.get("path").and_then(Value::as_str).unwrap_or("");
        let rendered = format!("{base}{path}");

        let mut query: Vec<(String, String)> = Vec::new();
        if let Some((_, raw)) = rendered.split_once('?') {
            for pair in raw.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    query.push((key.to_string(), value.to_string()));
                }
            }
        }
        if let Some(overrides) = definition.get("query").and_then(Value::as_object) {
            for (key, value) in overrides {
                let value = match value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                query.retain(|(k, _)| k != key);
                query.push((key.clone(), value));
            }
        }
        query.sort_by(|a, b| a.0.cmp(&b.0));

        let base_url = rendered.split('?').next().unwrap_or("").to_string();
        if query.is_empty() {
            return Ok(base_url);
        }
        let query = query
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        Ok(format!("{base_url}?{query}"))
    }
}

/// Route authoring.
pub struct Route;

impl Route {
    /// Build a route.
    pub fn make(input: Value) -> Value {
        input
    }

    /// Merge a patch into a route.
    pub fn with(base: Value, patch: Value) -> LlmResult<Value> {
        let mut route = match base {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        let patch = match patch {
            Value::Object(map) => map,
            _ => Map::new(),
        };

        if let Some(id) = patch.get("id") {
            route.insert("id".into(), id.clone());
        }
        if let Some(auth) = patch.get("auth") {
            route.insert("auth".into(), auth.clone());
        }
        if let Some(provider) = patch.get("provider") {
            route.insert("provider".into(), provider.clone());
        }
        if let Some(endpoint) = patch.get("endpoint") {
            let merged =
                merge_json_records(&[route.get("endpoint").cloned(), Some(endpoint.clone())])
                    .unwrap_or(Value::Null);
            route.insert("endpoint".into(), merged);
        }

        // Merge headers plus the remaining default fields into `defaults`.
        let mut defaults = route
            .get("defaults")
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()));
        let base_headers = route.remove("headers");
        let mut headers = base_headers
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        if let Some(patch_headers) = patch.get("headers").and_then(Value::as_object) {
            for (key, value) in patch_headers {
                headers.insert(key.clone(), value.clone());
            }
        }
        if !headers.is_empty() {
            defaults["headers"] = Value::Object(headers);
        }
        for key in ["generation", "providerOptions", "limits", "http"] {
            if let Some(value) = patch.get(key) {
                defaults[key] = value.clone();
            }
        }
        if !defaults.as_object().map(Map::is_empty).unwrap_or(true) {
            route.insert("defaults".into(), defaults);
        }
        Ok(Value::Object(route))
    }

    /// Build a model bound to the route.
    pub fn model(route: Value, input: Value) -> LlmResult<Value> {
        let mut model = match input {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        let route_id = route.get("id").cloned().unwrap_or(Value::Null);
        model.insert("route".into(), json!({ "id": route_id }));
        if let Some(provider) = route.get("provider") {
            model.entry("provider").or_insert(provider.clone());
        }
        if let Some(base_url) = route.get("endpoint").and_then(|e| e.get("baseURL")) {
            model.entry("endpoint").or_insert_with(|| json!({}));
            model["endpoint"]["baseURL"] = base_url.clone();
        }
        Ok(Value::Object(model))
    }
}

/// Protocol authoring.
pub struct Protocol;

impl Protocol {
    /// Build a protocol definition.
    pub fn make(input: Value) -> Value {
        input
    }
}

/// Auth authoring.
pub struct Auth;

impl Auth {
    /// Bearer auth.
    pub fn bearer(token: &str) -> Value {
        json!({ "kind": "bearer", "token": token })
    }

    /// A static header value.
    pub fn header(name: &str, value: &str) -> Value {
        json!({ "kind": "header", "name": name, "value": value })
    }

    /// A set of static headers.
    pub fn headers(map: Value) -> Value {
        json!({ "kind": "headers", "headers": map })
    }

    /// A config-backed credential.
    pub fn config(key: &str) -> Value {
        json!({ "kind": "config", "key": key })
    }

    /// A literal credential value.
    pub fn value(value: &str) -> Value {
        json!({ "kind": "value", "value": value })
    }

    /// Disable auth.
    pub fn none() -> Value {
        json!({ "kind": "none" })
    }

    /// Bearer auth rendered into a custom header.
    pub fn bearer_header(name: &str, token: &str) -> Value {
        json!({ "kind": "bearer-header", "name": name, "token": token })
    }

    /// A config-backed credential rendered as bearer auth.
    pub fn config_bearer(key: &str) -> Value {
        json!({ "kind": "config-bearer", "key": key })
    }

    /// Try the primary auth, falling back to the secondary.
    pub fn or_else(primary: Value, fallback: Value) -> Value {
        json!({ "kind": "or-else", "primary": primary, "fallback": fallback })
    }

    /// Compose two auth values in sequence.
    pub fn and_then(first: Value, second: Value) -> Value {
        json!({ "kind": "and-then", "first": first, "second": second })
    }

    /// Render an auth value into a custom header.
    pub fn render_header(auth: Value, name: &str) -> Value {
        json!({ "kind": "render-header", "auth": auth, "name": name })
    }

    /// Apply auth to a request input, returning the rendered headers.
    pub fn apply(auth: Value, input: Value) -> LlmResult<Value> {
        let env = input.get("env").cloned().unwrap_or_else(|| json!({}));
        let mut headers = input
            .get("headers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let rendered = render_auth(&auth, &env)?;
        for (key, value) in rendered {
            headers.insert(key, value);
        }
        Ok(Value::Object(headers))
    }
}

fn credential(auth: &Value, env: &Value) -> Option<Value> {
    match auth.get("kind").and_then(Value::as_str) {
        Some("value") => auth.get("value").cloned(),
        Some("config") => {
            let key = auth.get("key").and_then(Value::as_str)?;
            env.get(key).cloned()
        }
        Some("or-else") => credential(auth.get("primary").unwrap_or(&Value::Null), env)
            .or_else(|| credential(auth.get("fallback").unwrap_or(&Value::Null), env)),
        Some("bearer") => auth
            .get("token")
            .and_then(Value::as_str)
            .map(|token| Value::String(format!("Bearer {token}"))),
        _ => None,
    }
}

fn render_auth(auth: &Value, env: &Value) -> LlmResult<Map<String, Value>> {
    let mut headers = Map::new();
    match auth.get("kind").and_then(Value::as_str) {
        Some("none") => {}
        Some("bearer") => {
            let token = auth.get("token").and_then(Value::as_str).unwrap_or("");
            headers.insert(
                "authorization".into(),
                Value::String(format!("Bearer {token}")),
            );
        }
        Some("bearer-header") => {
            let name = auth.get("name").and_then(Value::as_str).unwrap_or("");
            let token = auth.get("token").and_then(Value::as_str).unwrap_or("");
            headers.insert(name.into(), Value::String(format!("Bearer {token}")));
        }
        Some("header") => {
            let name = auth.get("name").and_then(Value::as_str).unwrap_or("");
            let value = auth.get("value").cloned().unwrap_or(Value::Null);
            headers.insert(name.into(), value);
        }
        Some("headers") => {
            if let Some(map) = auth.get("headers").and_then(Value::as_object) {
                for (key, value) in map {
                    headers.insert(key.clone(), value.clone());
                }
            }
        }
        Some("config-bearer") => {
            let key = auth.get("key").and_then(Value::as_str).unwrap_or("");
            let value = env
                .get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| LlmError::InvalidRequest(format!("missing credential {key}")))?;
            headers.insert(
                "authorization".into(),
                Value::String(format!("Bearer {value}")),
            );
        }
        Some("render-header") => {
            let inner = auth.get("auth").unwrap_or(&Value::Null);
            let name = auth.get("name").and_then(Value::as_str).unwrap_or("");
            let value = credential(inner, env)
                .ok_or_else(|| LlmError::InvalidRequest("missing credential".into()))?;
            headers.insert(name.into(), value);
        }
        Some("or-else") => {
            let primary = auth.get("primary").unwrap_or(&Value::Null);
            match render_auth(primary, env) {
                Ok(rendered) if !rendered.is_empty() => headers = rendered,
                _ => {
                    headers = render_auth(auth.get("fallback").unwrap_or(&Value::Null), env)?;
                }
            }
        }
        Some("and-then") => {
            let first = render_auth(auth.get("first").unwrap_or(&Value::Null), env)?;
            let second = render_auth(auth.get("second").unwrap_or(&Value::Null), env)?;
            headers.extend(first);
            headers.extend(second);
        }
        _ => return Err(LlmError::InvalidRequest("unknown auth kind".into())),
    }
    Ok(headers)
}

/// Prepared request surfaced by `LLMClient::prepare`.
#[derive(Debug, Clone)]
pub struct Prepared {
    /// Resolved route id.
    pub route: String,
    /// Resolved protocol id.
    pub protocol: String,
    /// Resolved model descriptor.
    pub model: Value,
    /// Wire body.
    pub body: Value,
    /// Optional transport metadata.
    pub metadata: Option<Value>,
}

/// A completed generation.
#[derive(Debug, Clone)]
pub struct Response {
    /// Normalised event stream.
    pub events: Vec<Value>,
    /// Assembled assistant message.
    pub message: Value,
    /// Token accounting.
    pub usage: Value,
    /// Terminal finish reason.
    pub finish_reason: Option<String>,
    /// Concatenated assistant text.
    pub text: String,
    /// Concatenated reasoning text.
    pub reasoning: String,
    /// Assembled tool calls.
    pub tool_calls: Vec<Value>,
}

/// Raw executor response.
#[derive(Debug, Clone)]
pub struct ExecutedResponse {
    /// HTTP status.
    pub status: u16,
    /// Response body.
    pub body: String,
}

thread_local! {
    static INJECTED_RESPONSE: RefCell<Option<Value>> = const { RefCell::new(None) };
}

/// Test-only transport injection.
pub mod testing {
    use super::INJECTED_RESPONSE;
    use serde_json::Value;

    /// Queue a raw HTTP response (`{ status, body, headers }`) for the next call.
    pub fn push_response(response: Value) {
        INJECTED_RESPONSE.with(|slot| *slot.borrow_mut() = Some(response));
    }
}

pub(crate) fn take_injected() -> Option<Value> {
    INJECTED_RESPONSE.with(|slot| slot.borrow_mut().take())
}

/// The LLM client service.
pub struct LLMClient;

impl LLMClient {
    /// Prepare the wire body for a request.
    pub fn prepare(request: Json) -> LlmResult<Prepared> {
        crate::engine::prepare(request)
    }

    /// Generate a completion.
    pub fn generate(request: Json) -> LlmResult<Response> {
        let events = Self::stream(request)?;
        crate::engine::reduce_response(events)
    }

    /// Stream normalised events.
    pub fn stream(request: Json) -> LlmResult<Vec<Value>> {
        crate::engine::stream(request)
    }

    /// Service constructor.
    pub fn service() -> LlmResult<Value> {
        Ok(json!({ "service": "llm-client" }))
    }

    /// Layer constructor.
    pub fn layer() -> LlmResult<Value> {
        Ok(json!({ "layer": "llm-client" }))
    }
}

/// The request executor service.
pub struct RequestExecutor;

impl RequestExecutor {
    /// Execute an HTTP request with retry/classification.
    pub fn execute(input: Json) -> LlmResult<ExecutedResponse> {
        crate::executor::execute(input)
    }

    /// Service constructor.
    pub fn service() -> LlmResult<Value> {
        Ok(json!({ "service": "request-executor" }))
    }

    /// Layer constructor.
    pub fn layer() -> LlmResult<Value> {
        Ok(json!({ "layer": "request-executor" }))
    }
}

/// The WebSocket executor service.
pub struct WebSocketExecutor;

impl WebSocketExecutor {
    /// Adopt an already-open WebSocket.
    pub fn from_web_socket(_socket: Json, _input: Json) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("websocket executor"))
    }
}

pub(crate) use take_injected as take_injected_response;
