//! Session tool-progress projector (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/tool-progress.ts`: a started tool input projects a
//! `running` state with empty structured/content, a progress event updates
//! structured/content while keeping `running`, a success settles `completed`
//! with its structured/content, a failure keeps the last progress
//! structured/content and records the error, and progress, success and failure
//! events are durable. The Database/EventV2/SessionProjector wiring is replaced
//! by a pure tool-state projector.

use serde_json::{json, Value};

/// A projected assistant tool state.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolState {
    /// `running`, `completed` or `error`.
    pub status: String,
    /// Structured result.
    pub structured: Value,
    /// Durable content.
    pub content: Value,
    /// Error, when failed.
    pub error: Option<Value>,
}

/// A durable tool lifecycle event.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolEvent {
    /// The tool input started.
    Started,
    /// The tool reported progress.
    Progress {
        /// Structured progress.
        structured: Value,
        /// Durable content.
        content: Value,
    },
    /// The tool succeeded.
    Success {
        /// Structured result.
        structured: Value,
        /// Durable content.
        content: Value,
    },
    /// The tool failed.
    Failed {
        /// Error value.
        error: Value,
    },
}

/// Apply a tool event to the previous state.
pub fn apply(state: Option<&ToolState>, event: &ToolEvent) -> ToolState {
    match event {
        ToolEvent::Started => ToolState {
            status: "running".to_string(),
            structured: json!({}),
            content: json!([]),
            error: None,
        },
        ToolEvent::Progress {
            structured,
            content,
        } => ToolState {
            status: "running".to_string(),
            structured: structured.clone(),
            content: content.clone(),
            error: None,
        },
        ToolEvent::Success {
            structured,
            content,
        } => ToolState {
            status: "completed".to_string(),
            structured: structured.clone(),
            content: content.clone(),
            error: None,
        },
        ToolEvent::Failed { error } => ToolState {
            status: "error".to_string(),
            structured: state
                .map(|state| state.structured.clone())
                .unwrap_or_else(|| json!({})),
            content: state
                .map(|state| state.content.clone())
                .unwrap_or_else(|| json!([])),
            error: Some(error.clone()),
        },
    }
}

/// Whether an event type is durably settled.
pub fn is_durable(event_type: &str) -> bool {
    matches!(
        event_type,
        "session.next.tool.progress.1"
            | "session.next.tool.success.1"
            | "session.next.tool.failed.1"
    )
}
