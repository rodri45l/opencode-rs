//! Bounded tool output storage.
//!
//! Ports the observable behaviour of `packages/core/src/tool-output-store.ts`:
//! oversized text is written to one managed file and replaced with a bounded
//! preview, structured-only output is bounded via JSON, native media is
//! preserved without a settlement limit, and cleanup removes expired files.

use std::path::PathBuf;

use serde_json::Value;

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
    pub const MAX_BYTES: usize = 20_000;

    /// Create a store rooted at `root`.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Bound an output, writing managed files as needed.
    pub fn bound(
        &self,
        _session_id: &str,
        _tool_call_id: &str,
        _output: ToolOutput,
    ) -> CoreResult<BoundOutput> {
        let _ = &self.root;
        Err(CoreError::NotImplemented(
            "tool_output_store::ToolOutputStore::bound",
        ))
    }

    /// The effective configured limits.
    pub fn limits(&self) -> CoreResult<ToolOutputLimits> {
        Err(CoreError::NotImplemented(
            "tool_output_store::ToolOutputStore::limits",
        ))
    }

    /// Remove expired managed files.
    pub fn cleanup(&self) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "tool_output_store::ToolOutputStore::cleanup",
        ))
    }
}
