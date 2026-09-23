//! Session runner LLM event publisher.
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/runner/publish-llm-event.ts`: a local tool success
//! serializes media base64 exactly once and drops the compatibility `result`,
//! a provider-executed success keeps `result`, a binary failure publishes a
//! failed event and no success event, step finish records a settlement without
//! publishing a step ended event.

use serde_json::{json, Value};

use crate::CoreResult;

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
        self.events.clone()
    }

    /// The latest step settlement, if a step finished.
    pub fn step_settlement(&self) -> Option<Value> {
        self.step_settlement.clone()
    }

    /// Publish a tool call.
    pub fn publish_tool_call(&mut self, id: &str, name: &str, input: Value) -> CoreResult<()> {
        self.events.push(PublishedEvent {
            event_type: "session.next.tool.called.1".into(),
            data: json!({
                "sessionID": self.session_id,
                "callID": id,
                "tool": name,
                "input": input,
                "provider": { "executed": false },
            }),
        });
        Ok(())
    }

    /// Publish a tool success.
    pub fn publish_tool_result(&mut self, id: &str, spec: ToolResultSpec) -> CoreResult<()> {
        let mut data = serde_json::Map::new();
        data.insert("sessionID".into(), json!(self.session_id));
        data.insert("callID".into(), json!(id));
        data.insert("structured".into(), spec.structured);
        data.insert("content".into(), spec.content);
        data.insert("outputPaths".into(), json!([]));
        data.insert(
            "provider".into(),
            json!({ "executed": spec.provider_executed }),
        );
        if spec.provider_executed {
            if let Some(result) = spec.result {
                data.insert("result".into(), result);
            }
        }
        self.events.push(PublishedEvent {
            event_type: "session.next.tool.success.1".into(),
            data: Value::Object(data),
        });
        Ok(())
    }

    /// Publish a tool failure.
    pub fn publish_tool_failure(&mut self, id: &str, error: Value) -> CoreResult<()> {
        self.events.push(PublishedEvent {
            event_type: "session.next.tool.failed.1".into(),
            data: json!({
                "sessionID": self.session_id,
                "callID": id,
                "error": error,
                "provider": { "executed": false },
            }),
        });
        Ok(())
    }

    /// Publish a step start.
    pub fn publish_step_start(&mut self, index: u32) -> CoreResult<()> {
        self.events.push(PublishedEvent {
            event_type: "session.next.step.started.2".into(),
            data: json!({
                "sessionID": self.session_id,
                "agent": self.agent,
                "modelID": self.model_id,
                "providerID": self.provider_id,
                "step": index,
            }),
        });
        Ok(())
    }

    /// Publish a step finish, recording the settlement.
    pub fn publish_step_finish(&mut self, _index: u32, reason: &str) -> CoreResult<()> {
        if self.step_settlement.is_some() {
            return Err(crate::CoreError::Invalid(
                "duplicate step finish".to_string(),
            ));
        }
        self.step_settlement = Some(json!({
            "finish": reason,
            "tokens": { "input": 0, "output": 0, "reasoning": 0, "cache": { "read": 0, "write": 0 } },
        }));
        Ok(())
    }

    /// Decode a durable tool success payload, including legacy `result`.
    pub fn decode_success_data(value: &Value) -> CoreResult<Value> {
        let mut decoded = serde_json::Map::new();
        if let Some(call_id) = value.get("callID") {
            decoded.insert("callID".into(), call_id.clone());
        }
        if let Some(structured) = value.get("structured") {
            decoded.insert("structured".into(), structured.clone());
        }
        if let Some(content) = value.get("content") {
            decoded.insert("content".into(), content.clone());
        }
        if let Some(result) = value.get("result") {
            decoded.insert("result".into(), result.clone());
        }
        if let Some(provider) = value.get("provider") {
            decoded.insert("provider".into(), provider.clone());
        }
        Ok(Value::Object(decoded))
    }
}
