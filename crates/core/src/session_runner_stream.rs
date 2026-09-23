//! Session runner stream/policy helpers (re-derived behavioural subset).
//!
//! Ports the pure subset of `packages/core/src/session/runner.ts`: session prompt
//! cache keys are bounded to 64 characters, interleaved assistant text blocks
//! stay separate, duplicate streamed text starts and tool-input deltas before
//! their start are rejected, streamed raw tool input parses into the called
//! input, an agent's configured final step forces a text response and steering
//! input resets the allowance, and provider errors project as terminal assistant
//! step failures. The Database/EventV2/projector, provider stream, tool execution
//! and compaction/steering runtimes are replaced by pure helpers.

use std::collections::HashMap;
use std::fmt;

use serde_json::Value;

/// A session-runner stream failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    /// A text block was started twice.
    DuplicateTextStart {
        /// Block id.
        id: String,
    },
    /// A tool-input delta arrived before its start.
    ToolInputDeltaBeforeStart {
        /// Call id.
        id: String,
    },
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTextStart { id } => f.write_str(&duplicate_text_start_message(id)),
            Self::ToolInputDeltaBeforeStart { id } => {
                f.write_str(&tool_input_delta_before_start_message(id))
            }
        }
    }
}

impl std::error::Error for StreamError {}

/// Bound a session prompt cache key to 64 characters.
pub fn prompt_cache_key(session_id: &str) -> String {
    session_id
        .strip_prefix("ses_")
        .unwrap_or(session_id)
        .chars()
        .take(64)
        .collect()
}

/// The duplicate-text-start message.
pub fn duplicate_text_start_message(id: &str) -> String {
    format!("Duplicate text start: {id}")
}

/// The tool-input-delta-before-start message.
pub fn tool_input_delta_before_start_message(id: &str) -> String {
    format!("Tool input delta before start: {id}")
}

/// A streamed assistant text block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    /// Block id.
    pub id: String,
    /// Accumulated text.
    pub text: String,
}

/// An interleaved streamed text accumulator.
#[derive(Debug, Default)]
pub struct TextStream {
    blocks: Vec<TextBlock>,
    open: HashMap<String, usize>,
}

impl TextStream {
    /// Create an empty stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a text block.
    pub fn text_start(&mut self, id: &str) -> Result<(), StreamError> {
        if self.open.contains_key(id) {
            return Err(StreamError::DuplicateTextStart { id: id.to_string() });
        }
        self.blocks.push(TextBlock {
            id: id.to_string(),
            text: String::new(),
        });
        self.open.insert(id.to_string(), self.blocks.len() - 1);
        Ok(())
    }

    /// Append a delta to an open text block.
    pub fn text_delta(&mut self, id: &str, text: &str) -> Result<(), StreamError> {
        match self.open.get(id) {
            Some(index) => {
                self.blocks[*index].text.push_str(text);
                Ok(())
            }
            None => Err(StreamError::ToolInputDeltaBeforeStart { id: id.to_string() }),
        }
    }

    /// End a text block.
    pub fn text_end(&mut self, id: &str) -> Result<(), StreamError> {
        self.open.remove(id);
        Ok(())
    }

    /// The accumulated blocks in start order.
    pub fn blocks(&self) -> Vec<TextBlock> {
        self.blocks.clone()
    }
}

/// A streamed provider tool-input buffer.
#[derive(Debug, Default)]
pub struct ToolInputStream {
    raw: String,
    started: Option<String>,
    called: HashMap<String, Value>,
}

impl ToolInputStream {
    /// Create an empty stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start buffering a tool input.
    pub fn input_start(&mut self, id: &str, _name: &str) -> Result<(), StreamError> {
        self.started = Some(id.to_string());
        self.raw.clear();
        Ok(())
    }

    /// Append a raw tool-input delta.
    pub fn input_delta(&mut self, id: &str, _name: &str, text: &str) -> Result<(), StreamError> {
        if self.started.as_deref() != Some(id) {
            return Err(StreamError::ToolInputDeltaBeforeStart { id: id.to_string() });
        }
        self.raw.push_str(text);
        Ok(())
    }

    /// End a tool input, parsing the accumulated raw JSON.
    pub fn input_end(&mut self, id: &str, _name: &str) -> Result<(), StreamError> {
        if self.started.as_deref() == Some(id) {
            let parsed = serde_json::from_str(&self.raw).unwrap_or(Value::Null);
            self.called.insert(id.to_string(), parsed);
        }
        Ok(())
    }

    /// The parsed input for a completed tool call.
    pub fn called_input(&self, id: &str) -> Option<Value> {
        self.called.get(id).cloned()
    }

    /// The accumulated raw tool input.
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

/// The agent step allowance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepAllowance {
    /// Maximum steps.
    pub max_steps: u32,
    /// Steps consumed.
    pub used: u32,
}

impl StepAllowance {
    /// Create an allowance with `max_steps`.
    pub fn new(max_steps: u32) -> Self {
        Self { max_steps, used: 0 }
    }

    /// Whether the next step must respond with text only.
    pub fn force_text(&self) -> bool {
        self.used + 1 >= self.max_steps
    }

    /// Consume one step.
    pub fn consume(&mut self) {
        self.used += 1;
    }

    /// Reset the allowance when steering input promotes.
    pub fn reset_on_steer(&mut self) {
        self.used = 0;
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
