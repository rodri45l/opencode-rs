//! Rust client for the opencode server.
//!
//! Used by the CLI (`attach`/`run`) and by the conformance harness that compares
//! this server against the reference implementation.

use futures::{Stream, StreamExt};
use opencode_schema::EventEnvelope;
use serde::Deserialize;

/// Client error.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// Transport or HTTP error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// A malformed SSE frame.
    #[error("malformed event stream: {0}")]
    Protocol(String),
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
