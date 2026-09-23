//! Question service.
//!
//! Ports the observable behaviour of `packages/core/src/question.ts`: `ask`
//! registers a pending request with an ascending `que_` id, `list` returns the
//! pending requests, `reply` settles a request, `reject` removes it, and an
//! unknown request id is a not-found error.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{CoreError, CoreResult};

static COUNTER: AtomicU64 = AtomicU64::new(0);

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
        session_id: &str,
        questions: Vec<QuestionInfo>,
    ) -> CoreResult<QuestionRequest> {
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        let request = QuestionRequest {
            id: format!("que_{sequence:012}"),
            session_id: session_id.to_string(),
            questions,
        };
        self.requests.insert(request.id.clone(), request.clone());
        Ok(request)
    }

    /// List pending requests.
    pub fn list(&self) -> CoreResult<Vec<QuestionRequest>> {
        Ok(self.requests.values().cloned().collect())
    }

    /// Answer a pending request.
    pub fn reply(&mut self, request_id: &str, _answers: Vec<Vec<String>>) -> CoreResult<()> {
        if self.requests.remove(request_id).is_none() {
            return Err(CoreError::Invalid(format!(
                "question request not found: {request_id}"
            )));
        }
        Ok(())
    }

    /// Reject a pending request.
    pub fn reject(&mut self, request_id: &str) -> CoreResult<()> {
        if self.requests.remove(request_id).is_none() {
            return Err(CoreError::Invalid(format!(
                "question request not found: {request_id}"
            )));
        }
        Ok(())
    }
}
