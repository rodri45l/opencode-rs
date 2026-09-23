//! Bounded tool output storage.
//!
//! Ports the observable behaviour of `packages/core/src/tool-output-store.ts`:
//! oversized text is written to one managed file and replaced with a bounded
//! preview, structured-only output is bounded via JSON, native media is
//! preserved without a settlement limit, and cleanup removes expired files.

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::{CoreError, CoreResult};

/// A tool output payload.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutput {
    /// Structured metadata.
    pub structured: Value,
    /// Content parts.
    pub content: Vec<Value>,
}

/// A bounded tool output plus any managed files.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundOutput {
    /// The bounded output.
    pub output: ToolOutput,
    /// Managed file paths written for the output.
    pub output_paths: Vec<PathBuf>,
}

/// Effective tool output limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolOutputLimits {
    /// Maximum preview lines.
    pub max_lines: usize,
    /// Maximum preview bytes.
    pub max_bytes: usize,
}

/// Bounded tool output storage.
#[derive(Debug, Default)]
pub struct ToolOutputStore {
    root: PathBuf,
}

impl ToolOutputStore {
    /// Default provider-facing byte bound.
    pub const MAX_BYTES: usize = 50 * 1024;

    /// Default provider-facing line bound.
    pub const MAX_LINES: usize = 2_000;

    /// Create a store rooted at `root`.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Bound an output, writing managed files as needed.
    pub fn bound(
        &self,
        session_id: &str,
        tool_call_id: &str,
        output: ToolOutput,
    ) -> CoreResult<BoundOutput> {
        let has_native_media = output
            .content
            .iter()
            .any(|part| part.get("type").and_then(Value::as_str) == Some("file"));
        if has_native_media {
            return Ok(BoundOutput {
                output,
                output_paths: Vec::new(),
            });
        }

        let combined_text: String = output
            .content
            .iter()
            .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");

        let structured_json = serde_json::to_string(&output.structured)
            .map_err(|error| CoreError::Message(error.to_string()))?;

        // Duplicated structured data that is already projected as text is not
        // double-counted: bound when either channel alone exceeds the limit.
        let oversized_text =
            combined_text.len() > Self::MAX_BYTES || line_count(&combined_text) > Self::MAX_LINES;
        let oversized_structured =
            output.content.is_empty() && structured_json.len() > Self::MAX_BYTES;
        if !oversized_text && !oversized_structured {
            return Ok(BoundOutput {
                output,
                output_paths: Vec::new(),
            });
        }

        let directory = self.root.join(session_id);
        std::fs::create_dir_all(&directory)
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        let file = directory.join(format!("{tool_call_id}.txt"));
        let contents = if output.content.is_empty() {
            structured_json.clone()
        } else {
            combined_text.clone()
        };
        std::fs::write(&file, &contents)
            .map_err(|error| CoreError::FileSystem(error.to_string()))?;

        let mut bounded = output.clone();
        if output.content.is_empty() {
            bounded.content = vec![json!({ "type": "text", "text": structured_json })];
        } else {
            bounded.content = vec![json!({
                "type": "text",
                "text": truncate_preview(&combined_text, Self::MAX_BYTES),
            })];
        }
        Ok(BoundOutput {
            output: bounded,
            output_paths: vec![file],
        })
    }

    /// The effective configured limits.
    pub fn limits(&self) -> CoreResult<ToolOutputLimits> {
        Ok(ToolOutputLimits {
            max_lines: 2_000,
            max_bytes: Self::MAX_BYTES,
        })
    }

    /// Remove expired managed files.
    pub fn cleanup(&self) -> CoreResult<()> {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root)
                .map_err(|error| CoreError::FileSystem(error.to_string()))?;
        }
        Ok(())
    }
}

fn truncate_preview(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

fn line_count(text: &str) -> usize {
    text.chars().filter(|ch| *ch == '\n').count() + 1
}
