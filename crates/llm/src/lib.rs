//! Provider abstraction and streaming adapters.
//!
//! This crate is a from-scratch port of `packages/llm`. The port proceeds
//! test-first: the reference tests under `tests/` pin observable behaviour
//! (request shaping, streaming chunk parsing, tool-call assembly, usage/cost)
//! while the adapters are stubbed with typed [`LlmError::NotImplemented`]
//! errors.

#![forbid(unsafe_code)]

pub mod api;
pub mod cache_policy;
pub mod error;
pub mod protocols;
pub mod providers;
pub mod route;
pub mod schema;
pub mod shared;
pub mod tool;
pub mod tool_schema;
pub mod tool_stream;

/// The universal wire value used across the crate's public API.
pub type Json = serde_json::Value;

pub use api::LLM;
pub use cache_policy::apply_cache_policy;
pub use error::{LlmError, LlmResult};
pub use providers::Provider;
pub use route::{
    Auth, Endpoint, ExecutedResponse, LLMClient, Prepared, Protocol, RequestExecutor, Response,
    Route, WebSocketExecutor,
};
pub use schema::{
    CacheHint, ContentPart, LLMEvent, LLMRequest, LLMResponse, Message, Model, ToolCallPart,
    ToolChoice, ToolDefinition, ToolResultPart, Usage,
};
pub use shared::ProviderShared;
pub use tool::{Tool, ToolContent, ToolExecuteContext, ToolFailure, ToolOutput, ToolRuntime};
pub use tool_schema::ToolSchemaProjection;
pub use tool_stream::ToolStream;
