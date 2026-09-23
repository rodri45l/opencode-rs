//! Edit tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/edit.ts`: the tool
//! is registered as `edit`, accepts `path`, `oldString`, `newString`, and
//! `replaceAll`, rejects no-op/empty/missing/ambiguous exact replacements with
//! fixed messages, replaces one or all exact occurrences, reports replacements
//! and a diff, and returns `Unable to edit <path>` when approval is denied. The
//! `ToolRegistry`, `FileMutation`, BOM/CRLF preservation, and conditional-commit
//! wiring are dropped; the pure replacement decisions remain.

use crate::{CoreError, CoreResult};

/// The outcome of an exact replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditOutcome {
    /// Number of replacements performed.
    pub replacements: usize,
    /// The resulting content.
    pub content: String,
}

/// The edit tool.
#[derive(Debug, Default)]
pub struct EditTool;

impl EditTool {
    /// The tool name.
    pub const NAME: &'static str = "edit";

    /// The locked input schema property names.
    pub fn schema_keys() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_edit::EditTool::schema_keys",
        ))
    }

    /// Apply an exact replacement, returning the count and new content.
    pub fn apply(
        _content: &str,
        _old_string: &str,
        _new_string: &str,
        _replace_all: bool,
    ) -> CoreResult<EditOutcome> {
        Err(CoreError::NotImplemented("tool_edit::EditTool::apply"))
    }

    /// The success message for an edit.
    pub fn success_message(
        _resource: &str,
        _replacements: usize,
        _diff: &str,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_edit::EditTool::success_message",
        ))
    }

    /// The failure message when an edit is denied or fails.
    pub fn failure_message(_path: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_edit::EditTool::failure_message",
        ))
    }
}
