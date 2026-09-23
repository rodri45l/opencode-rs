//! Session runner LLM event publisher.
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/runner/publish-llm-event.ts`: a local tool success
//! serializes media base64 exactly once and drops the compatibility `result`,
//! a provider-executed success keeps `result`, a binary failure publishes a
//! failed event and no success event, step finish records a settlement without
//! publishing a step ended event.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A tool success payload.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResultSpec {
    /// Structured result content.
    pub structured: Value,
    /// Durable content parts.
    pub content: Value,
    /// Compatibility result carried for provider-executed tools.
    pub result: Option<Value>,
    /// Whether the provider executed the tool.
    pub provider_executed: bool,
}

/// A published event.
#[derive(Debug, Clone, PartialEq)]
pub struct PublishedEvent {
    /// Versioned durable event type.
    pub event_type: String,
    /// Event data.
    pub data: Value,
}

/// Publishes session runner lifecycle events.
#[derive(Debug)]
pub struct LlmEventPublisher {
    session_id: String,
    agent: String,
    model_id: String,
    provider_id: String,
    events: Vec<PublishedEvent>,
    step_settlement: Option<Value>,
}

impl LlmEventPublisher {
    /// Create a publisher scoped to a session/agent/model.
    pub fn new(
        session_id: impl Into<String>,
        agent: impl Into<String>,
        model_id: impl Into<String>,
        provider_id: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            agent: agent.into(),
            model_id: model_id.into(),
            provider_id: provider_id.into(),
            events: Vec::new(),
            step_settlement: None,
        }
    }

    /// Events published so far.
    pub fn published(&self) -> Vec<PublishedEvent> {
        let _ = (
            &self.session_id,
            &self.agent,
            &self.model_id,
            &self.provider_id,
        );
        self.events.clone()
    }

    /// The latest step settlement, if a step finished.
    pub fn step_settlement(&self) -> Option<Value> {
        self.step_settlement.clone()
    }

    /// Publish a tool call.
    pub fn publish_tool_call(&mut self, _id: &str, _name: &str, _input: Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::publish_tool_call",
        ))
    }

    /// Publish a tool success.
    pub fn publish_tool_result(&mut self, _id: &str, _spec: ToolResultSpec) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::publish_tool_result",
        ))
    }

    /// Publish a tool failure.
    pub fn publish_tool_failure(&mut self, _id: &str, _error: Value) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::publish_tool_failure",
        ))
    }

    /// Publish a step start.
    pub fn publish_step_start(&mut self, _index: u32) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::publish_step_start",
        ))
    }

    /// Publish a step finish, recording the settlement.
    pub fn publish_step_finish(&mut self, _index: u32, _reason: &str) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::publish_step_finish",
        ))
    }

    /// Decode a durable tool success payload, including legacy `result`.
    pub fn decode_success_data(_value: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "session_runner_tool_events::LlmEventPublisher::decode_success_data",
        ))
    }
}
