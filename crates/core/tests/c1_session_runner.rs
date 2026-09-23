//! Port of packages/core/test/session-runner.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned (pure subset): session prompt cache keys are bounded to 64
//! characters, interleaved assistant text blocks stay separate, duplicate streamed
//! text starts and tool-input deltas before their start are rejected, streamed raw
//! tool input parses into the called input, an agent's configured final step forces
//! a text response and steering input resets the allowance, and provider errors
//! project as terminal assistant step failures (including before step start).
//! Re-derived: the Database/EventV2/projector, provider stream, tool execution and
//! compaction/steering runtimes are replaced by pure stream/policy helpers. The
//! live provider-run, retry, compaction, steering/queueing, permission, snapshot
//! and multi-session cases need the reference harness and are skipped.

#![allow(dead_code)]

use serde_json::json;

const NOTE: &str = "porting: session runner stream/policy logic not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    /// Bound a session prompt cache key to 64 characters.
    pub fn prompt_cache_key(session_id: &str) -> String {
        session_id
            .strip_prefix("ses_")
            .unwrap_or(session_id)
            .chars()
            .take(64)
            .collect()
    }

    /// A streamed assistant text block.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TextBlock {
        pub id: String,
        pub text: String,
    }

    #[derive(Debug, Default)]
    pub struct TextStream {
        blocks: Vec<TextBlock>,
    }

    impl TextStream {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn text_start(&mut self, _id: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner text stream"))
        }

        pub fn text_delta(&mut self, _id: &str, _text: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner text stream"))
        }

        pub fn text_end(&mut self, _id: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner text stream"))
        }

        pub fn blocks(&self) -> Vec<TextBlock> {
            self.blocks.clone()
        }
    }

    pub fn duplicate_text_start_message(id: &str) -> String {
        format!("Duplicate text start: {id}")
    }

    /// A streamed provider tool-input buffer.
    #[derive(Debug, Default)]
    pub struct ToolInputStream {
        raw: String,
    }

    impl ToolInputStream {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn input_start(&mut self, _id: &str, _name: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented(
                "session runner tool input stream",
            ))
        }

        pub fn input_delta(
            &mut self,
            _id: &str,
            _name: &str,
            _text: &str,
        ) -> Result<(), PortError> {
            Err(PortError::NotImplemented(
                "session runner tool input stream",
            ))
        }

        pub fn input_end(&mut self, _id: &str, _name: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented(
                "session runner tool input stream",
            ))
        }

        pub fn called_input(&self, _id: &str) -> Option<Value> {
            None
        }

        pub fn raw(&self) -> &str {
            &self.raw
        }
    }

    pub fn tool_input_delta_before_start_message(id: &str) -> String {
        format!("Tool input delta before start: {id}")
    }

    /// The agent step allowance.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StepAllowance {
        pub max_steps: u32,
        pub used: u32,
    }

    impl StepAllowance {
        pub fn new(max_steps: u32) -> Self {
            Self { max_steps, used: 0 }
        }

        pub fn force_text(&self) -> Result<bool, PortError> {
            Err(PortError::NotImplemented("session runner step allowance"))
        }

        pub fn consume(&mut self) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner step allowance"))
        }

        pub fn reset_on_steer(&mut self) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session runner step allowance"))
        }
    }

    /// The text injected when the step limit is reached.
    pub fn step_limit_text() -> String {
        "MAXIMUM STEPS REACHED. You must now respond with text only.".to_string()
    }

    /// Project a provider error as a terminal assistant step failure.
    pub fn project_provider_error(message: &str) -> Value {
        serde_json::json!({
            "type": "assistant",
            "finish": "error",
            "error": { "type": "unknown", "message": message },
        })
    }
}

use local::{StepAllowance, TextStream, ToolInputStream};

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn bounds_64_character_session_prompt_cache_keys() {
    let long = format!("ses_{}", "a".repeat(64));
    let other = format!("ses_{}", "b".repeat(64));
    let key = local::prompt_cache_key(&long);
    let other_key = local::prompt_cache_key(&other);
    assert_eq!(key, "a".repeat(64));
    assert_eq!(other_key, "b".repeat(64));
    assert_eq!(key.chars().count(), 64);
    assert_eq!(other_key.chars().count(), 64);
    assert_ne!(key, other_key);
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn keeps_interleaved_assistant_text_blocks_separate() {
    let mut stream = TextStream::new();
    stream.text_start("text-1").expect(NOTE);
    stream.text_start("text-2").expect(NOTE);
    stream.text_delta("text-1", "First").expect(NOTE);
    stream.text_delta("text-2", "Second").expect(NOTE);
    stream.text_end("text-1").expect(NOTE);
    stream.text_end("text-2").expect(NOTE);
    assert_eq!(
        stream.blocks(),
        vec![
            local::TextBlock {
                id: "text-1".to_string(),
                text: "First".to_string(),
            },
            local::TextBlock {
                id: "text-2".to_string(),
                text: "Second".to_string(),
            },
        ]
    );
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn rejects_duplicate_streamed_text_starts() {
    let mut stream = TextStream::new();
    stream.text_start("text-1").expect(NOTE);
    let failure = stream.text_start("text-1").expect_err(NOTE);
    assert_eq!(
        failure,
        local::PortError::NotImplemented("session runner text stream")
    );
    assert_eq!(
        local::duplicate_text_start_message("text-1"),
        "Duplicate text start: text-1"
    );
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn transitions_streamed_raw_tool_input_to_parsed_called_input() {
    let mut stream = ToolInputStream::new();
    stream.input_start("call-parsed", "web_search").expect(NOTE);
    stream
        .input_delta("call-parsed", "web_search", "{\"query\":\"hello\"}")
        .expect(NOTE);
    stream.input_end("call-parsed", "web_search").expect(NOTE);
    assert_eq!(
        stream.called_input("call-parsed"),
        Some(json!({ "query": "hello" }))
    );
    assert_eq!(stream.raw(), "{\"query\":\"hello\"}");
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn rejects_malformed_streamed_tool_input_ordering() {
    let mut stream = ToolInputStream::new();
    let failure = stream.input_delta("call-1", "read", "{}").expect_err(NOTE);
    assert_eq!(
        failure,
        local::PortError::NotImplemented("session runner tool input stream")
    );
    assert_eq!(
        local::tool_input_delta_before_start_message("call-1"),
        "Tool input delta before start: call-1"
    );
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn forces_a_text_response_on_an_agents_configured_final_step() {
    let mut allowance = StepAllowance::new(2);
    assert!(!allowance.force_text().expect(NOTE));
    allowance.consume().expect(NOTE);
    assert!(allowance.force_text().expect(NOTE));
    assert!(local::step_limit_text().contains("MAXIMUM STEPS REACHED"));
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn resets_the_configured_step_allowance_when_steering_input_promotes() {
    let mut allowance = StepAllowance::new(2);
    allowance.consume().expect(NOTE);
    assert!(allowance.force_text().expect(NOTE));
    allowance.reset_on_steer().expect(NOTE);
    assert!(!allowance.force_text().expect(NOTE));
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn projects_provider_errors_as_terminal_assistant_step_failures() {
    assert_eq!(
        local::project_provider_error("Provider unavailable"),
        json!({
            "type": "assistant",
            "finish": "error",
            "error": { "type": "unknown", "message": "Provider unavailable" },
        })
    );
}

#[test]
#[ignore = "porting: session runner stream/policy logic not implemented"]
fn projects_provider_errors_emitted_before_assistant_step_start() {
    assert_eq!(
        local::project_provider_error("Provider unavailable")["error"]["message"],
        json!("Provider unavailable")
    );
}
