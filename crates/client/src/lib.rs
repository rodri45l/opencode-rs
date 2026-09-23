//! Rust client for the opencode server.
//!
//! Used by the CLI (`attach`/`run`) and by the conformance harness that compares
//! this server against the reference implementation.
//!
//! Phase 1 exposes the health probe and the SSE event stream. The typed HTTP
//! surface (grouped operations, session flow, declared errors) is declared here
//! as stubs returning [`ClientError::NotImplemented`] so the ported client
//! tests compile and stay red until the transport lands.

use futures::{future::BoxFuture, stream::BoxStream, Stream, StreamExt};
use opencode_schema::{EventEnvelope, LocationRef};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Client error.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Transport or HTTP error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// A malformed SSE frame.
    #[error("malformed event stream: {0}")]
    Protocol(String),
    /// A client method that has not been implemented yet.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    /// A declared `SessionNotFoundError` returned by the server.
    #[error("session not found: {session_id}")]
    SessionNotFound { session_id: String },
    /// A declared `UnauthorizedError` returned by the server.
    #[error("unauthorized: {message}")]
    Unauthorized { message: String },
    /// An unexpected non-success HTTP status.
    #[error("http status {status}: {body}")]
    HttpStatus { status: u16, body: String },
}

/// One outgoing HTTP request handed to a [`Transport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportRequest {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
}

/// One HTTP response returned by a [`Transport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportResponse {
    pub status: u16,
    pub body: String,
}

/// A stream of response body chunks returned by [`Transport::stream`].
pub type BodyStream = BoxStream<'static, Result<Vec<u8>, ClientError>>;

/// The HTTP seam the typed client is built on. The production implementation
/// wraps `reqwest`; tests inject a recording fake.
pub trait Transport: Send + Sync {
    /// Perform one request.
    fn request(
        &self,
        request: TransportRequest,
    ) -> BoxFuture<'static, Result<TransportResponse, ClientError>>;

    /// Perform one request and stream the response body as it arrives.
    ///
    /// The default implementation buffers through [`Transport::request`], which
    /// is correct for finite responses and for injected test transports. The
    /// production transport overrides this so long-lived `text/event-stream`
    /// responses deliver frames without waiting for the server to close.
    fn stream(
        &self,
        request: TransportRequest,
    ) -> BoxFuture<'static, Result<BodyStream, ClientError>> {
        let response = self.request(request);
        Box::pin(async move {
            let response = response.await?;
            if !(200..300).contains(&response.status) {
                return Err(declared_error(&response));
            }
            let stream = futures::stream::once(async move { Ok(response.body.into_bytes()) });
            Ok(Box::pin(stream) as BodyStream)
        })
    }
}

struct HttpTransport {
    http: reqwest::Client,
}

impl Transport for HttpTransport {
    fn request(
        &self,
        request: TransportRequest,
    ) -> BoxFuture<'static, Result<TransportResponse, ClientError>> {
        let http = self.http.clone();
        Box::pin(async move {
            let method = reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|err| ClientError::Protocol(err.to_string()))?;
            let mut builder = http.request(method, request.url);
            if let Some(body) = request.body {
                builder = builder
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body);
            }
            let response = builder.send().await?;
            let status = response.status().as_u16();
            let body = response.text().await?;
            Ok(TransportResponse { status, body })
        })
    }

    fn stream(
        &self,
        request: TransportRequest,
    ) -> BoxFuture<'static, Result<BodyStream, ClientError>> {
        let http = self.http.clone();
        Box::pin(async move {
            let method = reqwest::Method::from_bytes(request.method.as_bytes())
                .map_err(|err| ClientError::Protocol(err.to_string()))?;
            let mut builder = http.request(method, request.url);
            if let Some(body) = request.body {
                builder = builder
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body);
            }
            let response = builder.send().await?;
            let status = response.status().as_u16();
            if !(200..300).contains(&status) {
                let body = response.text().await.unwrap_or_default();
                return Err(declared_error(&TransportResponse { status, body }));
            }
            let stream = response
                .bytes_stream()
                .map(|chunk| chunk.map(|bytes| bytes.to_vec()).map_err(ClientError::Http));
            Ok(Box::pin(stream) as BodyStream)
        })
    }
}

fn declared_error(response: &TransportResponse) -> ClientError {
    let parsed: Option<serde_json::Value> = serde_json::from_str(&response.body).ok();
    let tag = parsed
        .as_ref()
        .and_then(|value| value.get("_tag"))
        .and_then(serde_json::Value::as_str);
    match tag {
        Some("SessionNotFoundError") => ClientError::SessionNotFound {
            session_id: parsed
                .as_ref()
                .and_then(|value| value.get("sessionID"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        Some("UnauthorizedError") => ClientError::Unauthorized {
            message: parsed
                .as_ref()
                .and_then(|value| value.get("message"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unauthorized")
                .to_string(),
        },
        _ => ClientError::HttpStatus {
            status: response.status,
            body: response.body.clone(),
        },
    }
}

fn decode_response<T: DeserializeOwned>(response: TransportResponse) -> Result<T, ClientError> {
    if !(200..300).contains(&response.status) {
        return Err(declared_error(&response));
    }
    serde_json::from_str(&response.body)
        .map_err(|err| ClientError::Protocol(format!("invalid response json: {err}")))
}

fn expect_ok(response: TransportResponse) -> Result<(), ClientError> {
    if (200..300).contains(&response.status) {
        Ok(())
    } else {
        Err(declared_error(&response))
    }
}

/// True when `err` is a declared `SessionNotFoundError`.
pub fn is_session_not_found_error(err: &ClientError) -> bool {
    matches!(err, ClientError::SessionNotFound { .. })
}

/// True when `err` is a declared `UnauthorizedError`.
pub fn is_unauthorized_error(err: &ClientError) -> bool {
    matches!(err, ClientError::Unauthorized { .. })
}

/// Health response.
#[derive(Debug, Clone, Deserialize)]
pub struct Health {
    pub healthy: bool,
}

/// An HTTP + SSE client for one server instance.
#[derive(Clone)]
pub struct Client {
    base_url: String,
    transport: Arc<dyn Transport>,
}

impl Client {
    /// Create a client for `base_url` (e.g. `http://127.0.0.1:8081`).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_transport(
            base_url,
            Arc::new(HttpTransport {
                http: reqwest::Client::new(),
            }),
        )
    }

    /// Create a client backed by an injected [`Transport`].
    pub fn with_transport(base_url: impl Into<String>, transport: Arc<dyn Transport>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            transport,
        }
    }

    /// The configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn send(&self, request: TransportRequest) -> Result<TransportResponse, ClientError> {
        self.transport.request(request).await
    }

    /// `GET /api/health`.
    pub async fn health(&self) -> Result<Health, ClientError> {
        let url = format!("{}/api/health", self.base_url);
        let response = self
            .send(TransportRequest {
                method: "GET".into(),
                url,
                body: None,
            })
            .await?;
        decode_response(response)
    }

    /// `GET /api/event` as a stream of decoded envelopes.
    ///
    /// The stream stays open until the server closes it or the caller drops it.
    pub fn events(&self) -> impl Stream<Item = Result<EventEnvelope, ClientError>> + '_ {
        let url = format!("{}/api/event", self.base_url);
        let transport = self.transport.clone();
        async_stream::try_stream! {
            let mut chunks = transport
                .stream(TransportRequest { method: "GET".into(), url, body: None })
                .await?;
            let mut buffer: Vec<u8> = Vec::new();
            while let Some(chunk) = chunks.next().await {
                buffer.extend_from_slice(&chunk?);
                while let Some(index) = find_frame_end(&buffer) {
                    let frame: Vec<u8> = buffer.drain(..index + 2).collect();
                    if let Some(event) = parse_frame_bytes(&frame)? {
                        yield event;
                    }
                }
            }
        }
    }

    /// The API groups this client exposes, in reference declaration order.
    ///
    /// Mirrors `Object.keys(OpenCode.make(...))` in the reference Promise client.
    pub fn groups(&self) -> Result<Vec<Group>, ClientError> {
        Ok(vec![
            Group {
                name: "health",
                operations: &["get"],
            },
            Group {
                name: "location",
                operations: &["list", "get"],
            },
            Group {
                name: "agents",
                operations: &["list"],
            },
            Group {
                name: "sessions",
                operations: &[],
            },
            Group {
                name: "messages",
                operations: &["list"],
            },
            Group {
                name: "models",
                operations: &["list"],
            },
            Group {
                name: "providers",
                operations: &["list"],
            },
            Group {
                name: "integrations",
                operations: &[
                    "list",
                    "get",
                    "connectKey",
                    "connectOauth",
                    "attemptStatus",
                    "attemptComplete",
                    "attemptCancel",
                ],
            },
            Group {
                name: "credentials",
                operations: &[],
            },
            Group {
                name: "permissions",
                operations: &[],
            },
            Group {
                name: "files",
                operations: &["list", "find"],
            },
            Group {
                name: "commands",
                operations: &["list"],
            },
            Group {
                name: "skills",
                operations: &["list"],
            },
            Group {
                name: "events",
                operations: &[],
            },
            Group {
                name: "ptys",
                operations: &["list", "create", "get", "update", "remove"],
            },
            Group {
                name: "questions",
                operations: &[],
            },
            Group {
                name: "references",
                operations: &[],
            },
            Group {
                name: "projectCopies",
                operations: &[],
            },
        ])
    }

    /// The `sessions` group handle.
    pub fn sessions(&self) -> Sessions<'_> {
        Sessions { client: self }
    }
}

/// A named API group and its operation names, in reference order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub operations: &'static [&'static str],
}

/// Decode a complete `text/event-stream` body into envelopes.
///
/// Frames without a `data:` line (comments/heartbeats) are skipped; a frame
/// whose data is not a valid envelope fails the whole decode.
pub fn decode_event_stream(body: &str) -> Result<Vec<EventEnvelope>, ClientError> {
    let mut events = Vec::new();
    let mut rest = body;
    while let Some(index) = rest.find("\n\n") {
        let (frame, remainder) = rest.split_at(index + 2);
        rest = remainder;
        if let Some(event) = parse_frame(frame)? {
            events.push(event);
        }
    }
    Ok(events)
}

fn parse_frame(frame: &str) -> Result<Option<EventEnvelope>, ClientError> {
    let mut data = String::new();
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.trim_start());
        }
    }
    if data.is_empty() {
        return Ok(None);
    }
    let event = serde_json::from_str(&data)
        .map_err(|err| ClientError::Protocol(format!("invalid event json: {err}")))?;
    Ok(Some(event))
}

fn find_frame_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(2).position(|window| window == b"\n\n")
}

fn parse_frame_bytes(frame: &[u8]) -> Result<Option<EventEnvelope>, ClientError> {
    let text = std::str::from_utf8(frame)
        .map_err(|err| ClientError::Protocol(format!("invalid event stream utf8: {err}")))?;
    parse_frame(text)
}

// ---------------------------------------------------------------------------
// Typed HTTP surface (stubs).
//
// These types mirror the wire DTOs the reference generated client decodes. The
// methods are unimplemented; they exist so the ported tests compile and fail
// with a typed `NotImplemented` error until the transport is wired.
// ---------------------------------------------------------------------------

/// The `sessions` API group.
pub struct Sessions<'a> {
    pub client: &'a Client,
}

/// Options for `sessions.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListInput {
    pub limit: Option<i64>,
    pub order: Option<String>,
}

/// Input for `sessions.create`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInput {
    pub location: LocationRef,
}

/// Input for `sessions.prompt`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInput {
    pub session_id: String,
    pub prompt: Prompt,
    pub resume: bool,
}

/// Input for `sessions.history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryInput {
    pub session_id: String,
    pub after: Option<i64>,
    pub limit: Option<i64>,
}

/// Input for `sessions.events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEventsInput {
    pub session_id: String,
    pub after: Option<i64>,
}

/// A prompt payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub text: String,
}

/// A model reference (`{ id, providerID }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRef {
    pub id: String,
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

/// Cache token accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheUsage {
    pub read: i64,
    pub write: i64,
}

/// Token accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache: CacheUsage,
}

/// Creation/update timestamps (epoch millis).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeInfo {
    pub created: i64,
    pub updated: i64,
}

/// A session wire projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(rename = "projectID")]
    pub project_id: String,
    pub cost: f64,
    pub tokens: TokenUsage,
    pub time: TimeInfo,
    pub title: String,
    pub location: LocationRef,
}

/// A list cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageCursor {
    pub next: String,
}

/// A page of sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionPage {
    pub data: Vec<Session>,
    pub cursor: PageCursor,
}

/// A session running state (`{ type }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveStatus {
    #[serde(rename = "type")]
    pub kind: String,
}

/// A prompt admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Admission {
    #[serde(rename = "admittedSeq")]
    pub admitted_seq: i64,
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub prompt: Prompt,
    pub delivery: String,
    #[serde(rename = "timeCreated")]
    pub time_created: i64,
}

/// A decoded session message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub time: TimeInfo,
    pub model: ModelRef,
}

/// A page of durable session history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryPage {
    pub data: Vec<EventEnvelope>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

impl Sessions<'_> {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.client.base_url, path)
    }

    async fn get_raw(
        &self,
        method: &str,
        url: String,
        body: Option<String>,
    ) -> Result<TransportResponse, ClientError> {
        self.client
            .send(TransportRequest {
                method: method.to_string(),
                url,
                body,
            })
            .await
    }

    fn json_body<T: Serialize>(value: &T) -> Result<String, ClientError> {
        serde_json::to_string(value).map_err(|err| ClientError::Protocol(err.to_string()))
    }

    /// `GET /api/session/:sessionID`.
    pub async fn get(&self, session_id: &str) -> Result<Session, ClientError> {
        let url = self.url(&format!("/api/session/{session_id}"));
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }

    /// `GET /api/session`.
    pub async fn list(&self, input: ListInput) -> Result<SessionPage, ClientError> {
        let mut query: Vec<String> = Vec::new();
        if let Some(limit) = input.limit {
            query.push(format!("limit={limit}"));
        }
        if let Some(order) = input.order {
            query.push(format!("order={order}"));
        }
        let url = if query.is_empty() {
            self.url("/api/session")
        } else {
            format!("{}?{}", self.url("/api/session"), query.join("&"))
        };
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }

    /// `GET /api/session/active`.
    pub async fn active(&self) -> Result<HashMap<String, ActiveStatus>, ClientError> {
        let url = self.url("/api/session/active");
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }

    /// `POST /api/session`.
    pub async fn create(&self, input: CreateInput) -> Result<Session, ClientError> {
        let url = self.url("/api/session");
        let response = self
            .get_raw("POST", url, Some(Self::json_body(&input)?))
            .await?;
        decode_response(response)
    }

    /// `POST /api/session/:sessionID/agent`.
    pub async fn switch_agent(&self, session_id: &str, agent: &str) -> Result<(), ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/agent"));
        let body = Self::json_body(&serde_json::json!({ "agent": agent }))?;
        let response = self.get_raw("POST", url, Some(body)).await?;
        expect_ok(response)
    }

    /// `POST /api/session/:sessionID/model`.
    pub async fn switch_model(&self, session_id: &str, model: ModelRef) -> Result<(), ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/model"));
        let response = self
            .get_raw("POST", url, Some(Self::json_body(&model)?))
            .await?;
        expect_ok(response)
    }

    /// `POST /api/session/:sessionID/prompt`.
    pub async fn prompt(&self, input: PromptInput) -> Result<Admission, ClientError> {
        let url = self.url(&format!("/api/session/{}/prompt", input.session_id));
        let body = Self::json_body(&serde_json::json!({
            "prompt": input.prompt,
            "resume": input.resume,
        }))?;
        let response = self.get_raw("POST", url, Some(body)).await?;
        decode_response(response)
    }

    /// `POST /api/session/:sessionID/compact`.
    pub async fn compact(&self, session_id: &str) -> Result<(), ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/compact"));
        let response = self.get_raw("POST", url, None).await?;
        expect_ok(response)
    }

    /// `POST /api/session/:sessionID/wait`.
    pub async fn wait(&self, session_id: &str) -> Result<(), ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/wait"));
        let response = self.get_raw("POST", url, None).await?;
        expect_ok(response)
    }

    /// `GET /api/session/:sessionID/context`.
    pub async fn context(&self, session_id: &str) -> Result<Vec<serde_json::Value>, ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/context"));
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }

    /// `GET /api/session/:sessionID/history`.
    pub async fn history(&self, input: HistoryInput) -> Result<HistoryPage, ClientError> {
        let mut query: Vec<String> = Vec::new();
        if let Some(limit) = input.limit {
            query.push(format!("limit={limit}"));
        }
        if let Some(after) = input.after {
            query.push(format!("after={after}"));
        }
        let base = self.url(&format!("/api/session/{}/history", input.session_id));
        let url = if query.is_empty() {
            base
        } else {
            format!("{}?{}", base, query.join("&"))
        };
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }

    /// `GET /api/session/:sessionID/event`.
    pub fn events(
        &self,
        input: SessionEventsInput,
    ) -> impl Stream<Item = Result<EventEnvelope, ClientError>> + '_ {
        let base = self.url(&format!("/api/session/{}/event", input.session_id));
        let url = match input.after {
            Some(after) => format!("{base}?after={after}"),
            None => base,
        };
        let transport = self.client.transport.clone();
        async_stream::try_stream! {
            let response = transport
                .request(TransportRequest { method: "GET".into(), url, body: None })
                .await?;
            if !(200..300).contains(&response.status) {
                Err(declared_error(&response))?;
            }
            for event in decode_event_stream(&response.body)? {
                yield event;
            }
        }
    }

    /// `POST /api/session/:sessionID/interrupt`.
    pub async fn interrupt(&self, session_id: &str) -> Result<(), ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/interrupt"));
        let response = self.get_raw("POST", url, None).await?;
        expect_ok(response)
    }

    /// `GET /api/session/:sessionID/message/:messageID`.
    pub async fn message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<SessionMessage, ClientError> {
        let url = self.url(&format!("/api/session/{session_id}/message/{message_id}"));
        let response = self.get_raw("GET", url, None).await?;
        decode_response(response)
    }
}
