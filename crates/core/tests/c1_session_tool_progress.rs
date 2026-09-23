//! Port of packages/core/test/session-tool-progress.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a started tool input projects a `running` state with empty
//! structured/content, a progress event updates structured/content while keeping
//! `running`, a success settles `completed` with its structured/content, a failure
//! keeps the last progress structured/content and records the error, and progress,
//! success and failure events are durable.
//! Re-derived: the Database/EventV2/SessionProjector wiring and the event-table
//! sequence assertions are replaced by a pure tool-state projector in
//! `opencode_core::session_tool_progress`.

use opencode_core::session_tool_progress::{apply, is_durable, ToolEvent, ToolState};
use serde_json::{json, Value};

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
fn projects_a_started_tool_input_as_running_with_empty_structured_and_content() {
    let state = apply(None, &ToolEvent::Started);
    assert_eq!(state.status, "running");
    assert_eq!(state.structured, json!({}));
    assert_eq!(state.content, json!([]));
}

#[test]
fn projects_durable_progress_while_keeping_running() {
    let state = apply(
        Some(&running()),
        &ToolEvent::Progress {
            structured: json!({ "phase": "checkpoint" }),
            content: content("saved"),
        },
    );
    assert_eq!(state.status, "running");
    assert_eq!(state.structured, json!({ "phase": "checkpoint" }));
    assert_eq!(state.content, content("saved"));
}

#[test]
fn settles_a_success_as_completed() {
    let state = apply(
        Some(&running()),
        &ToolEvent::Success {
            structured: json!({ "phase": "done" }),
            content: content("complete"),
        },
    );
    assert_eq!(state.status, "completed");
    assert_eq!(state.structured, json!({ "phase": "done" }));
    assert_eq!(state.content, content("complete"));
}

#[test]
fn keeps_the_last_progress_when_a_tool_fails() {
    let progress = apply(
        Some(&running()),
        &ToolEvent::Progress {
            structured: json!({ "phase": "checkpoint" }),
            content: content("before failure"),
        },
    );
    let failed = apply(
        Some(&progress),
        &ToolEvent::Failed {
            error: json!({ "type": "unknown", "message": "boom" }),
        },
    );
    assert_eq!(failed.status, "error");
    assert_eq!(failed.structured, json!({ "phase": "checkpoint" }));
    assert_eq!(failed.content, content("before failure"));
    assert_eq!(
        failed.error,
        Some(json!({ "type": "unknown", "message": "boom" }))
    );
}

#[test]
fn keeps_final_settlements_durable() {
    assert!(is_durable("session.next.tool.progress.1"));
    assert!(is_durable("session.next.tool.success.1"));
    assert!(is_durable("session.next.tool.failed.1"));
    assert!(!is_durable("session.next.tool.input.started.1"));
    assert!(!is_durable("session.next.tool.called.1"));
}
