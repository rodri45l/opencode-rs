//! Rust client for the opencode server.
//!
//! Used by the CLI (`attach`/`run`) and by the conformance harness that compares
//! this server against the reference implementation.
//!
//! Phase 1 exposes the health probe and the SSE event stream. The typed HTTP
//! surface (grouped operations, session flow, declared errors) is declared here
//! as stubs returning [`ClientError::NotImplemented`] so the ported client
//! tests compile and stay red until the transport lands.

use futures::{Stream, StreamExt};
use opencode_schema::{EventEnvelope, LocationRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

impl Client {
    /// Create a client for `base_url` (e.g. `http://127.0.0.1:8081`).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// The configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// `GET /api/health`.
    pub async fn health(&self) -> Result<Health, ClientError> {
        let url = format!("{}/api/health", self.base_url);
        Ok(self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// `GET /api/event` as a stream of decoded envelopes.
    ///
    /// The stream stays open until the server closes it or the caller drops it.
    pub fn events(&self) -> impl Stream<Item = Result<EventEnvelope, ClientError>> + '_ {
        let url = format!("{}/api/event", self.base_url);
        let http = self.http.clone();
        async_stream::try_stream! {
            let response = http.get(url).send().await?.error_for_status()?;
            let mut bytes = response.bytes_stream();
            let mut buffer = String::new();
            while let Some(chunk) = bytes.next().await {
                buffer.push_str(&String::from_utf8_lossy(&chunk?));
                while let Some(index) = buffer.find("\n\n") {
                    let frame: String = buffer.drain(..index + 2).collect();
                    if let Some(event) = parse_frame(&frame)? {
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
        Err(ClientError::NotImplemented("client groups"))
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
    /// `GET /api/session/:sessionID`.
    pub async fn get(&self, _session_id: &str) -> Result<Session, ClientError> {
        Err(ClientError::NotImplemented("sessions.get"))
    }

    /// `GET /api/session`.
    pub async fn list(&self, _input: ListInput) -> Result<SessionPage, ClientError> {
        Err(ClientError::NotImplemented("sessions.list"))
    }

    /// `GET /api/session/active`.
    pub async fn active(&self) -> Result<HashMap<String, ActiveStatus>, ClientError> {
        Err(ClientError::NotImplemented("sessions.active"))
    }

    /// `POST /api/session`.
    pub async fn create(&self, _input: CreateInput) -> Result<Session, ClientError> {
        Err(ClientError::NotImplemented("sessions.create"))
    }

    /// `POST /api/session/:sessionID/agent`.
    pub async fn switch_agent(&self, _session_id: &str, _agent: &str) -> Result<(), ClientError> {
        Err(ClientError::NotImplemented("sessions.switchAgent"))
    }

    /// `POST /api/session/:sessionID/model`.
    pub async fn switch_model(
        &self,
        _session_id: &str,
        _model: ModelRef,
    ) -> Result<(), ClientError> {
        Err(ClientError::NotImplemented("sessions.switchModel"))
    }

    /// `POST /api/session/:sessionID/prompt`.
    pub async fn prompt(&self, _input: PromptInput) -> Result<Admission, ClientError> {
        Err(ClientError::NotImplemented("sessions.prompt"))
    }

    /// `POST /api/session/:sessionID/compact`.
    pub async fn compact(&self, _session_id: &str) -> Result<(), ClientError> {
        Err(ClientError::NotImplemented("sessions.compact"))
    }

    /// `POST /api/session/:sessionID/wait`.
    pub async fn wait(&self, _session_id: &str) -> Result<(), ClientError> {
        Err(ClientError::NotImplemented("sessions.wait"))
    }

    /// `GET /api/session/:sessionID/context`.
    pub async fn context(&self, _session_id: &str) -> Result<Vec<serde_json::Value>, ClientError> {
        Err(ClientError::NotImplemented("sessions.context"))
    }

    /// `GET /api/session/:sessionID/history`.
    pub async fn history(&self, _input: HistoryInput) -> Result<HistoryPage, ClientError> {
        Err(ClientError::NotImplemented("sessions.history"))
    }

    /// `GET /api/session/:sessionID/event`.
    pub fn events(
        &self,
        _input: SessionEventsInput,
    ) -> impl Stream<Item = Result<EventEnvelope, ClientError>> + '_ {
        futures::stream::once(async { Err(ClientError::NotImplemented("sessions.events")) })
    }

    /// `POST /api/session/:sessionID/interrupt`.
    pub async fn interrupt(&self, _session_id: &str) -> Result<(), ClientError> {
        Err(ClientError::NotImplemented("sessions.interrupt"))
    }

    /// `GET /api/session/:sessionID/message/:messageID`.
    pub async fn message(
        &self,
        _session_id: &str,
        _message_id: &str,
    ) -> Result<SessionMessage, ClientError> {
        Err(ClientError::NotImplemented("sessions.message"))
    }
}
