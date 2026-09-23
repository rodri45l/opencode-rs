//! Route, protocol, endpoint, auth and client surfaces.

use serde_json::Value;

use crate::error::{LlmError, LlmResult};
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
    pub fn render(_definition: Value, _context: Value) -> LlmResult<String> {
        Err(LlmError::NotImplemented("endpoint render"))
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
    pub fn with(_base: Value, _patch: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("route with"))
    }

    /// Build a model bound to the route.
    pub fn model(_route: Value, _input: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("route model"))
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
        serde_json::json!({ "kind": "bearer", "token": token })
    }

    /// A static header value.
    pub fn header(name: &str, value: &str) -> Value {
        serde_json::json!({ "kind": "header", "name": name, "value": value })
    }

    /// A set of static headers.
    pub fn headers(map: Value) -> Value {
        serde_json::json!({ "kind": "headers", "headers": map })
    }

    /// A config-backed credential.
    pub fn config(key: &str) -> Value {
        serde_json::json!({ "kind": "config", "key": key })
    }

    /// A literal credential value.
    pub fn value(value: &str) -> Value {
        serde_json::json!({ "kind": "value", "value": value })
    }

    /// Disable auth.
    pub fn none() -> Value {
        serde_json::json!({ "kind": "none" })
    }

    /// Bearer auth rendered into a custom header.
    pub fn bearer_header(name: &str, token: &str) -> Value {
        serde_json::json!({ "kind": "bearer-header", "name": name, "token": token })
    }

    /// A config-backed credential rendered as bearer auth.
    pub fn config_bearer(key: &str) -> Value {
        serde_json::json!({ "kind": "config-bearer", "key": key })
    }

    /// Try the primary auth, falling back to the secondary.
    pub fn or_else(primary: Value, fallback: Value) -> Value {
        serde_json::json!({ "kind": "or-else", "primary": primary, "fallback": fallback })
    }

    /// Compose two auth values in sequence.
    pub fn and_then(first: Value, second: Value) -> Value {
        serde_json::json!({ "kind": "and-then", "first": first, "second": second })
    }

    /// Render an auth value into a custom header.
    pub fn render_header(auth: Value, name: &str) -> Value {
        serde_json::json!({ "kind": "render-header", "auth": auth, "name": name })
    }

    /// Apply auth to a request input, returning the rendered headers.
    pub fn apply(_auth: Value, _input: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("auth apply"))
    }
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

/// The LLM client service.
pub struct LLMClient;

impl LLMClient {
    /// Prepare the wire body for a request.
    pub fn prepare(_request: Json) -> LlmResult<Prepared> {
        Err(LlmError::NotImplemented("llm client prepare"))
    }

    /// Generate a completion.
    pub fn generate(_request: Json) -> LlmResult<Response> {
        Err(LlmError::NotImplemented("llm client generate"))
    }

    /// Stream normalised events.
    pub fn stream(_request: Json) -> LlmResult<Vec<Value>> {
        Err(LlmError::NotImplemented("llm client stream"))
    }

    /// Service constructor.
    pub fn service() -> LlmResult<Value> {
        Err(LlmError::NotImplemented("llm client service"))
    }

    /// Layer constructor.
    pub fn layer() -> LlmResult<Value> {
        Err(LlmError::NotImplemented("llm client layer"))
    }
}

/// The request executor service.
pub struct RequestExecutor;

impl RequestExecutor {
    /// Execute an HTTP request with retry/classification.
    pub fn execute(_request: Json) -> LlmResult<ExecutedResponse> {
        Err(LlmError::NotImplemented("request executor"))
    }

    /// Service constructor.
    pub fn service() -> LlmResult<Value> {
        Err(LlmError::NotImplemented("request executor service"))
    }

    /// Layer constructor.
    pub fn layer() -> LlmResult<Value> {
        Err(LlmError::NotImplemented("request executor layer"))
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
