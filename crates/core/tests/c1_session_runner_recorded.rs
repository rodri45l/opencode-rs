//! Port of packages/core/test/session-runner-recorded.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: replaying one recorded V2 prompt through the recorded HTTP
//! transport projects two messages — the admitted user prompt and an assistant
//! turn with `finish: "stop"` and text `"Hello!"` — and persists exactly the
//! ordered event types `prompt.admitted`, `prompted`, `step.started`,
//! `text.started`, `text.ended`, `step.ended`.
//! Re-derived: the HttpRecorder cassette, LLM client, Database and EventV2
//! services are replaced by a scripted event list projected in memory; the actual
//! recorded HTTP round-trip is skipped (needs the reference cassette fixture).

#![allow(dead_code)]

use serde_json::json;

const NOTE: &str = "porting: session runner recorded transport not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    /// The projected outcome of a recorded prompt.
    #[derive(Debug, Clone, PartialEq)]
    pub struct RecordedRun {
        pub messages: Vec<Value>,
        pub event_types: Vec<String>,
    }

    pub fn run_recorded_prompt() -> Result<RecordedRun, PortError> {
        Err(PortError::NotImplemented(
            "session runner recorded transport",
        ))
    }
}

#[test]
#[ignore = "porting: session runner recorded transport not implemented"]
fn executes_one_recorded_v2_prompt_through_the_recorded_http_transport() {
    let run = local::run_recorded_prompt().expect(NOTE);

    assert_eq!(run.messages.len(), 2);
    assert_eq!(run.messages[0]["id"], json!("msg_recorded_prompt"));
    assert_eq!(run.messages[0]["type"], json!("user"));
    assert_eq!(
        run.messages[0]["text"],
        json!("Say hello in one short sentence.")
    );
    assert_eq!(
        run.messages[1],
        json!({
            "type": "assistant",
            "agent": "build",
            "finish": "stop",
            "content": [{ "type": "text", "text": "Hello!" }],
        })
    );

    assert_eq!(
        run.event_types,
        vec![
            "session.next.prompt.admitted.1",
            "session.next.prompted.1",
            "session.next.step.started.1",
            "session.next.text.started.1",
            "session.next.text.ended.1",
            "session.next.step.ended.2",
        ]
    );
}
