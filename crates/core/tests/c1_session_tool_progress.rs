//! Port of packages/core/test/session-tool-progress.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a started tool input projects a `running` state with empty
//! structured/content, a progress event updates structured/content while keeping
//! `running`, a success settles `completed` with its structured/content, a failure
//! keeps the last progress structured/content and records the error, and progress,
//! success and failure events are durable.
//! Re-derived: the Database/EventV2/SessionProjector wiring and the event-table
//! sequence assertions are replaced by a pure tool-state projector; the durable
//! classification is checked against the versioned event type.

#![allow(dead_code)]

use serde_json::{json, Value};

const NOTE: &str = "porting: session tool progress projector not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    /// A projected assistant tool state.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ToolState {
        pub status: String,
        pub structured: Value,
        pub content: Value,
        pub error: Option<Value>,
    }

    /// A durable tool lifecycle event.
    #[derive(Debug, Clone, PartialEq)]
    pub enum ToolEvent {
        Started,
        Progress { structured: Value, content: Value },
        Success { structured: Value, content: Value },
        Failed { error: Value },
    }

    pub fn apply(_state: Option<&ToolState>, _event: &ToolEvent) -> Result<ToolState, PortError> {
        Err(PortError::NotImplemented("session tool progress projector"))
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
}

use local::{ToolEvent, ToolState};

fn content(text: &str) -> Value {
    json!([{ "type": "text", "text": text }])
}

fn running() -> ToolState {
    ToolState {
        status: "running".to_string(),
        structured: json!({}),
        content: json!([]),
        error: None,
    }
}

#[test]
#[ignore = "porting: session tool progress projector not implemented"]
fn projects_a_started_tool_input_as_running_with_empty_structured_and_content() {
    let state = local::apply(None, &ToolEvent::Started).expect(NOTE);
    assert_eq!(state.status, "running");
    assert_eq!(state.structured, json!({}));
    assert_eq!(state.content, json!([]));
}

#[test]
#[ignore = "porting: session tool progress projector not implemented"]
fn projects_durable_progress_while_keeping_running() {
    let state = local::apply(
        Some(&running()),
        &ToolEvent::Progress {
            structured: json!({ "phase": "checkpoint" }),
            content: content("saved"),
        },
    )
    .expect(NOTE);
    assert_eq!(state.status, "running");
    assert_eq!(state.structured, json!({ "phase": "checkpoint" }));
    assert_eq!(state.content, content("saved"));
}

#[test]
#[ignore = "porting: session tool progress projector not implemented"]
fn settles_a_success_as_completed() {
    let state = local::apply(
        Some(&running()),
        &ToolEvent::Success {
            structured: json!({ "phase": "done" }),
            content: content("complete"),
        },
    )
    .expect(NOTE);
    assert_eq!(state.status, "completed");
    assert_eq!(state.structured, json!({ "phase": "done" }));
    assert_eq!(state.content, content("complete"));
}

#[test]
#[ignore = "porting: session tool progress projector not implemented"]
fn keeps_the_last_progress_when_a_tool_fails() {
    let progress = local::apply(
        Some(&running()),
        &ToolEvent::Progress {
            structured: json!({ "phase": "checkpoint" }),
            content: content("before failure"),
        },
    )
    .expect(NOTE);
    let failed = local::apply(
        Some(&progress),
        &ToolEvent::Failed {
            error: json!({ "type": "unknown", "message": "boom" }),
        },
    )
    .expect(NOTE);
    assert_eq!(failed.status, "error");
    assert_eq!(failed.structured, json!({ "phase": "checkpoint" }));
    assert_eq!(failed.content, content("before failure"));
    assert_eq!(
        failed.error,
        Some(json!({ "type": "unknown", "message": "boom" }))
    );
}

#[test]
#[ignore = "porting: session tool progress projector not implemented"]
fn keeps_final_settlements_durable() {
    assert!(local::is_durable("session.next.tool.progress.1"));
    assert!(local::is_durable("session.next.tool.success.1"));
    assert!(local::is_durable("session.next.tool.failed.1"));
    assert!(!local::is_durable("session.next.tool.input.started.1"));
    assert!(!local::is_durable("session.next.tool.called.1"));
}
