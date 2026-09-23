//! Provider stream-error parsing and model status classification.
//!
//! Ports the observable behaviour of `packages/opencode/src/provider/error.ts`
//! (`parseStreamError`) and `packages/opencode/src/provider/model-status.ts`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A parsed provider stream error.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStreamError {
    /// Machine-readable kind: `api_error` or `context_overflow`.
    pub kind: String,
    /// Human-readable message.
    pub message: String,
    /// Whether retrying may succeed.
    pub is_retryable: bool,
    /// The response body that produced the error.
    pub response_body: String,
}

/// Parse a provider `{ type: "error", ... }` payload into a stream error.
///
/// Mirrors the fallback branch: unknown error codes become retryable
/// `api_error`s carrying the provider message.
pub fn parse_stream_error(input: &Value) -> Option<ParsedStreamError> {
    let body = normalize(input)?;
    if body.get("type").and_then(Value::as_str) != Some("error") {
        return None;
    }

    let message = body
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("Server error.")
        .to_string();

    Some(ParsedStreamError {
        kind: "api_error".to_string(),
        message,
        is_retryable: true,
        response_body: serde_json::to_string(&body).unwrap_or_default(),
    })
}

fn normalize(input: &Value) -> Option<Value> {
    match input {
        Value::String(raw) => serde_json::from_str::<Value>(raw).ok(),
        Value::Object(_) => Some(input.clone()),
        _ => None,
    }
}

/// Catalog status, a strict subset that never accepts `active`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogModelStatus {
    /// The model is deprecated in the catalog.
    Deprecated,
}

impl CatalogModelStatus {
    /// Parse the wire spelling, rejecting `active`.
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "deprecated" => Ok(Self::Deprecated),
            other => Err(if other == "active" {
                "active is a normalized status, not a catalog status"
            } else {
                "unknown catalog status"
            }),
        }
    }
}

/// Normalized provider model status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    /// The model is active.
    Active,
}

impl ModelStatus {
    /// Parse the wire spelling.
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "active" => Ok(Self::Active),
            _ => Err("unknown model status"),
        }
    }
}

/// Config provider model carrying its resolved status.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConfigProviderModel {
    /// Resolved status.
    pub status: ModelStatus,
}

/// Models.dev catalog entry; status is absent from catalog payloads.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ModelsDevModel {
    /// Catalog id.
    pub id: String,
    /// Optional status, never set by the catalog.
    #[serde(default)]
    pub status: Option<String>,
}

/// Normalized provider model carrying its status.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProviderModel {
    /// Model id.
    pub id: String,
    /// Provider id.
    #[serde(rename = "providerID")]
    pub provider_id: String,
    /// Resolved status.
    pub status: String,
}
