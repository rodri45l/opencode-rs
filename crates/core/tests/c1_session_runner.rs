//! Port of packages/core/test/session-runner.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned (pure subset): session prompt cache keys are bounded to 64
//! characters, interleaved assistant text blocks stay separate, duplicate streamed
//! text starts and tool-input deltas before their start are rejected, streamed raw
//! tool input parses into the called input, an agent's configured final step forces
//! a text response and steering input resets the allowance, and provider errors
//! project as terminal assistant step failures (including before step start).
//! Re-derived: the runtime is replaced by pure stream/policy helpers in
//! `opencode_core::session_runner_stream`. The live provider-run, retry,
//! compaction, steering/queueing, permission, snapshot and multi-session cases
//! need the reference harness and are skipped.

use opencode_core::session_runner_stream::{
    duplicate_text_start_message, project_provider_error, prompt_cache_key, step_limit_text,
    tool_input_delta_before_start_message, StepAllowance, TextBlock, TextStream, ToolInputStream,
};
use serde_json::json;

#[test]
fn bounds_64_character_session_prompt_cache_keys() {
    let long = format!("ses_{}", "a".repeat(64));
    let other = format!("ses_{}", "b".repeat(64));
    let key = prompt_cache_key(&long);
    let other_key = prompt_cache_key(&other);
    assert_eq!(key, "a".repeat(64));
    assert_eq!(other_key, "b".repeat(64));
    assert_eq!(key.chars().count(), 64);
    assert_eq!(other_key.chars().count(), 64);
    assert_ne!(key, other_key);
}

#[test]
fn keeps_interleaved_assistant_text_blocks_separate() {
    let mut stream = TextStream::new();
    stream.text_start("text-1").expect("start");
    stream.text_start("text-2").expect("start");
    stream.text_delta("text-1", "First").expect("delta");
    stream.text_delta("text-2", "Second").expect("delta");
    stream.text_end("text-1").expect("end");
    stream.text_end("text-2").expect("end");
    assert_eq!(
        stream.blocks(),
        vec![
            TextBlock {
                id: "text-1".to_string(),
                text: "First".to_string(),
            },
            TextBlock {
                id: "text-2".to_string(),
                text: "Second".to_string(),
            },
        ]
    );
}

#[test]
fn rejects_duplicate_streamed_text_starts() {
    let mut stream = TextStream::new();
    stream.text_start("text-1").expect("start");
    let failure = stream.text_start("text-1").expect_err("duplicate");
    assert_eq!(failure.to_string(), duplicate_text_start_message("text-1"));
    assert_eq!(
        duplicate_text_start_message("text-1"),
        "Duplicate text start: text-1"
    );
}

#[test]
fn transitions_streamed_raw_tool_input_to_parsed_called_input() {
    let mut stream = ToolInputStream::new();
    stream
        .input_start("call-parsed", "web_search")
        .expect("start");
    stream
        .input_delta("call-parsed", "web_search", "{\"query\":\"hello\"}")
        .expect("delta");
    stream.input_end("call-parsed", "web_search").expect("end");
    assert_eq!(
        stream.called_input("call-parsed"),
        Some(json!({ "query": "hello" }))
    );
    assert_eq!(stream.raw(), "{\"query\":\"hello\"}");
}

#[test]
fn rejects_malformed_streamed_tool_input_ordering() {
    let mut stream = ToolInputStream::new();
    let failure = stream
        .input_delta("call-1", "read", "{}")
        .expect_err("delta");
    assert_eq!(
        failure.to_string(),
        tool_input_delta_before_start_message("call-1")
    );
    assert_eq!(
        tool_input_delta_before_start_message("call-1"),
        "Tool input delta before start: call-1"
    );
}

#[test]
fn forces_a_text_response_on_an_agents_configured_final_step() {
    let mut allowance = StepAllowance::new(2);
    assert!(!allowance.force_text());
    allowance.consume();
    assert!(allowance.force_text());
    assert!(step_limit_text().contains("MAXIMUM STEPS REACHED"));
}

#[test]
fn resets_the_configured_step_allowance_when_steering_input_promotes() {
    let mut allowance = StepAllowance::new(2);
    allowance.consume();
    assert!(allowance.force_text());
    allowance.reset_on_steer();
    assert!(!allowance.force_text());
}

#[test]
fn projects_provider_errors_as_terminal_assistant_step_failures() {
    assert_eq!(
        project_provider_error("Provider unavailable"),
        json!({
            "type": "assistant",
            "finish": "error",
            "error": { "type": "unknown", "message": "Provider unavailable" },
        })
    );
}

#[test]
fn projects_provider_errors_emitted_before_assistant_step_start() {
    assert_eq!(
        project_provider_error("Provider unavailable")["error"]["message"],
        json!("Provider unavailable")
    );
}
