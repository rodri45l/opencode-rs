//! Question service.
//!
//! Ports the observable behaviour of `packages/core/src/question.ts`: `ask`
//! registers a pending request with an ascending `que_` id, `list` returns the
//! pending requests, `reply` settles a request, `reject` removes it, and an
//! unknown request id is a not-found error.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// A selectable question option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionOption {
    /// Option label.
    pub label: String,
    /// Option description.
    pub description: String,
}

/// A question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionInfo {
    /// The question text.
    pub question: String,
    /// Short header.
    pub header: String,
    /// Selectable options.
    pub options: Vec<QuestionOption>,
}

/// A pending question request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionRequest {
    /// Request id (`que_...`).
    pub id: String,
    /// Owning session id.
    pub session_id: String,
    /// The questions asked.
    pub questions: Vec<QuestionInfo>,
}

/// The question service.
#[derive(Debug, Default)]
pub struct QuestionV2 {
    requests: BTreeMap<String, QuestionRequest>,
}

impl QuestionV2 {
    /// Create an empty service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ask a question, registering a pending request.
    pub fn ask(
        &mut self,
        _session_id: &str,
        _questions: Vec<QuestionInfo>,
    ) -> CoreResult<QuestionRequest> {
        let _ = &self.requests;
        Err(CoreError::NotImplemented("question::QuestionV2::ask"))
    }

    /// List pending requests.
    pub fn list(&self) -> CoreResult<Vec<QuestionRequest>> {
        let _ = &self.requests;
        Err(CoreError::NotImplemented("question::QuestionV2::list"))
    }

    /// Answer a pending request.
    pub fn reply(&mut self, _request_id: &str, _answers: Vec<Vec<String>>) -> CoreResult<()> {
        let _ = &self.requests;
        Err(CoreError::NotImplemented("question::QuestionV2::reply"))
    }

    /// Reject a pending request.
    pub fn reject(&mut self, _request_id: &str) -> CoreResult<()> {
        let _ = &self.requests;
        Err(CoreError::NotImplemented("question::QuestionV2::reject"))
    }
}
