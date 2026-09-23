//! Tool-surface stubs for the tools whose tests land in this wave.
//!
//! The read/edit/apply_patch/glob/question/skill/lsp tools are not implemented
//! yet; each `execute` returns [`ToolError::NotImplemented`] so the ported
//! tests stay red without aborting the suite.

use crate::tools::{ToolContext, ToolError, ToolResult};
use serde_json::Value;

/// Arguments for [`ReadTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadArgs {
    /// File to read.
    pub file_path: String,
    /// 1-based line offset.
    pub offset: Option<u64>,
    /// Maximum lines to return.
    pub limit: Option<u64>,
}

/// The `read` tool.
#[derive(Debug, Default)]
pub struct ReadTool;

impl ReadTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Read a file, honouring offset/limit and emitting attachment metadata.
    pub fn execute(
        &self,
        _args: ReadArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::ReadTool::execute"))
    }
}

/// Arguments for [`EditTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditArgs {
    /// File to edit.
    pub file_path: String,
    /// Text to replace; empty creates a new file.
    pub old_string: String,
    /// Replacement text.
    pub new_string: String,
    /// Replace every occurrence instead of exactly one.
    pub replace_all: Option<bool>,
}

/// The `edit` tool.
#[derive(Debug, Default)]
pub struct EditTool;

impl EditTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Apply a single string replacement to a file.
    pub fn execute(
        &self,
        _args: EditArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::EditTool::execute"))
    }
}

/// Arguments for [`ApplyPatchTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchArgs {
    /// The freeform patch body.
    pub patch_text: String,
}

/// The `apply_patch` tool.
#[derive(Debug, Default)]
pub struct ApplyPatchTool;

impl ApplyPatchTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Apply a freeform patch against the instance directory.
    pub fn execute(
        &self,
        _args: ApplyPatchArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::ApplyPatchTool::execute"))
    }
}

/// Arguments for [`GlobTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobArgs {
    /// Glob pattern.
    pub pattern: String,
    /// Directory to search; defaults to the instance directory.
    pub path: Option<String>,
}

/// The `glob` tool.
#[derive(Debug, Default)]
pub struct GlobTool;

impl GlobTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// List files matching a glob pattern.
    pub fn execute(
        &self,
        _args: GlobArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::GlobTool::execute"))
    }
}

/// One selectable answer to a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionOption {
    /// Option label.
    pub label: String,
    /// Option description.
    pub description: String,
}

/// A question posed to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionSpec {
    /// The question text.
    pub question: String,
    /// Short header (the reference warns above 30 chars).
    pub header: String,
    /// Selectable options.
    pub options: Vec<QuestionOption>,
    /// Whether multiple options may be selected.
    pub multiple: Option<bool>,
}

/// Arguments for [`QuestionTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionArgs {
    /// Questions to ask.
    pub questions: Vec<QuestionSpec>,
}

/// The `question` tool.
#[derive(Debug, Default)]
pub struct QuestionTool;

impl QuestionTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Ask the user the supplied questions and wait for replies.
    pub fn execute(
        &self,
        _args: QuestionArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::QuestionTool::execute"))
    }
}

/// Arguments for [`SkillTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillArgs {
    /// Skill name.
    pub name: String,
}

/// The `skill` tool.
#[derive(Debug, Default)]
pub struct SkillTool;

impl SkillTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Load a skill's `SKILL.md` and reference files.
    pub fn execute(
        &self,
        _args: SkillArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::SkillTool::execute"))
    }
}

/// Arguments for [`LspTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspArgs {
    /// Operation name, e.g. `goToDefinition` or `workspaceSymbol`.
    pub operation: String,
    /// File the operation targets.
    pub file_path: String,
    /// 1-based line for position-based operations.
    pub line: Option<u64>,
    /// 1-based character for position-based operations.
    pub character: Option<u64>,
    /// Query for `workspaceSymbol`.
    pub query: Option<String>,
}

/// The `lsp` tool.
#[derive(Debug, Default)]
pub struct LspTool;

impl LspTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Run an LSP operation and format the result.
    pub fn execute(&self, _args: LspArgs, _ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::LspTool::execute"))
    }
}

/// The instance-relative metadata a tool attaches, kept for the ported tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncateLimits {
    /// Maximum lines before truncation.
    pub max_lines: usize,
    /// Maximum bytes before truncation.
    pub max_bytes: usize,
}

/// The default truncation limits exposed by the `truncate` tool.
pub fn truncate_limits() -> TruncateLimits {
    TruncateLimits {
        max_lines: 2000,
        max_bytes: 50 * 1024,
    }
}

/// Placeholder metadata value so callers can assert JSON shapes.
pub fn empty_metadata() -> Value {
    Value::Null
}
