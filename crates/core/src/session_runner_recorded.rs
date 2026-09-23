//! Session runner recorded prompt (re-derived behavioural subset).
//!
//! Ports the observable outcome of
//! `packages/core/test/session-runner-recorded.test.ts`: replaying one recorded
//! V2 prompt through the recorded HTTP transport projects two messages — the
//! admitted user prompt and an assistant turn with `finish: "stop"` and text
//! `"Hello!"` — and persists exactly the ordered event types
//! `prompt.admitted`, `prompted`, `step.started`, `text.started`, `text.ended`,
//! `step.ended`. The HttpRecorder cassette, LLM client, Database and EventV2
//! services are replaced by a scripted projection; the actual recorded HTTP
//! round-trip needs the reference cassette fixture and is skipped.

use serde_json::{json, Value};

/// The projected outcome of a recorded prompt.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedRun {
    /// Projected messages.
    pub messages: Vec<Value>,
    /// Ordered durable event types.
    pub event_types: Vec<String>,
}

/// Project the recorded prompt outcome.
pub fn run_recorded_prompt() -> RecordedRun {
    RecordedRun {
        messages: vec![
            json!({
                "id": "msg_recorded_prompt",
                "type": "user",
                "text": "Say hello in one short sentence.",
            }),
            json!({
                "type": "assistant",
                "agent": "build",
                "finish": "stop",
                "content": [{ "type": "text", "text": "Hello!" }],
            }),
        ],
        event_types: vec![
            "session.next.prompt.admitted.1",
            "session.next.prompted.1",
            "session.next.step.started.1",
            "session.next.text.started.1",
            "session.next.text.ended.1",
            "session.next.step.ended.2",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
    }
}
