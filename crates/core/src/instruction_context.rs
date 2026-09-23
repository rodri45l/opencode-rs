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
    pub fn render(_files: &[InstructionFile]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "instruction_context::InstructionContext::render",
        ))
    }
}
