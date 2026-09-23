//! Provider abstraction and streaming adapters.
//!
//! Placeholder for Phase 5. The goal is a single `Provider` trait with a
//! streaming interface, plus one adapter per upstream API family, each tested
//! against recorded cassettes from the reference `http-recorder`.

use opencode_schema::EventType;
use serde_json::Value;

/// A single model descriptor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
}

/// A provider request for one completion turn.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Value>,
    pub tools: Vec<Value>,
}

/// A normalised streaming chunk produced by a provider adapter.
#[derive(Debug, Clone)]
pub enum Chunk {
    /// Streamed assistant text.
    Text(String),
    /// Streamed reasoning text.
    Reasoning(String),
    /// A tool call became available.
    ToolCall {
        id: String,
        name: String,
        input: Value,
    },
    /// The provider reported terminal usage/cost.
    Finished { finish: String, cost: f64 },
}

/// A model provider (OpenAI-compatible, Anthropic, Google, ...).
pub trait Provider: Send + Sync + 'static {
    /// Stable provider id, matching `models.dev`.
    fn id(&self) -> &str;

    /// Models exposed by this provider.
    fn models(&self) -> Vec<ModelInfo>;

    /// Stream a completion. Implementations land in Phase 5.
    fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = Chunk> + Send>, LlmError>;
}

/// Placeholder error until adapters are implemented.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("not implemented")]
    NotImplemented,
    #[error("provider error: {0}")]
    Provider(String),
}

/// The event type emitted when a model list is refreshed.
pub const CATALOG_EVENT: EventType = EventType::CatalogUpdated;
