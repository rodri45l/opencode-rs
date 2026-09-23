//! Question tool (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/question.ts`: the
//! tool is registered as `question`, asserts the wildcard permission action and
//! does not invent tool ownership metadata, projects answered questions into the
//! fixed model-facing sentence (unanswered questions become `Unanswered`), and is
//! omitted entirely when a deny rule matches. The `QuestionV2`/`Permission`/
//! `ToolRegistry` wiring and fiber settlement are dropped; the pure projection
//! remains.

use crate::{CoreError, CoreResult};

/// A single question presented to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The question text.
    pub question: String,
    /// The short header.
    pub header: String,
    /// The selectable option labels.
    pub options: Vec<String>,
}

/// The question tool.
#[derive(Debug, Default)]
pub struct QuestionTool;

impl QuestionTool {
    /// The tool name.
    pub const NAME: &'static str = "question";

    /// The permission action constant.
    pub const ACTION: &'static str = "question";

    /// The permission action for the tool.
    pub fn permission_action() -> CoreResult<&'static str> {
        Err(CoreError::NotImplemented(
            "tool_question::QuestionTool::permission_action",
        ))
    }

    /// The permission resources for the tool.
    pub fn permission_resources() -> CoreResult<Vec<&'static str>> {
        Err(CoreError::NotImplemented(
            "tool_question::QuestionTool::permission_resources",
        ))
    }

    /// Whether the tool is omitted because the wildcard question action is denied.
    pub fn omitted_when_denied(_deny_wildcard_question: bool) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "tool_question::QuestionTool::omitted_when_denied",
        ))
    }

    /// The error output when the tool is denied.
    pub fn denied_message() -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_question::QuestionTool::denied_message",
        ))
    }

    /// Project the user's answers into the model-facing sentence.
    pub fn format_answers(_questions: &[Question], _answers: &[Vec<String>]) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "tool_question::QuestionTool::format_answers",
        ))
    }
}
