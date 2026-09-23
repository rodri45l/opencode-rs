//! Ambient instruction aggregation.
//!
//! Ports the observable behaviour of `packages/core/src/instruction-context.ts`:
//! global, package and project `AGENTS.md` files are rendered as one aggregate
//! system context, with empty files still counted as available context.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A discovered instruction file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionFile {
    /// Absolute path.
    pub path: AbsolutePath,
    /// File contents.
    pub content: String,
}

/// Instruction context rendering.
#[derive(Debug, Default)]
pub struct InstructionContext;

impl InstructionContext {
    /// Render discovered instruction files into one aggregate context.
    pub fn render(files: &[InstructionFile]) -> CoreResult<String> {
        if files.is_empty() {
            return Ok(String::new());
        }
        Ok(files
            .iter()
            .map(|file| format!("Instructions from: {}\n{}", file.path, file.content))
            .collect::<Vec<_>>()
            .join("\n\n"))
    }

    /// Render the update text when the discovered files replace prior ones.
    pub fn replace(files: &[InstructionFile]) -> CoreResult<String> {
        let rendered = Self::render(files)?;
        if rendered.is_empty() {
            Ok("Previously loaded instructions no longer apply.".to_string())
        } else {
            Ok(format!(
                "These instructions replace all previously loaded ambient instructions.\n\n{rendered}"
            ))
        }
    }

    /// The removal message for previously admitted instructions.
    pub fn removed_message() -> &'static str {
        "Previously loaded instructions no longer apply."
    }
}

/// A render error that would previously abort aggregation.
pub fn invalid_instructions(path: &str) -> CoreError {
    CoreError::Message(format!("invalid instructions file: {path}"))
}
