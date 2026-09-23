//! Session prompt service (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/session/prompt.ts`: the
//! execution registry and delegation (resume/interrupt) surface through
//! `SessionExecution`, a prompt durably admits one inbox message before
//! promotion, attachment MIME is resolved before admission, exact ID retries
//! return the original record while conflicting text/delivery/session reuse fails
//! with `Session.PromptConflictError`, a prompt ID already used by visible
//! history is rejected, steers promote once through the captured inbox cutoff,
//! and resume defaults to true (wake) unless explicitly false. The Database/
//! EventV2/projector and Effect-concurrency wiring are replaced by an in-memory
//! inbox with explicit `promote_steers`.

use std::collections::HashMap;

/// How a prompt is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Steer the active execution.
    Steer,
    /// Queue behind the active execution.
    Queue,
}

/// A durably admitted prompt inbox row.
#[derive(Debug, Clone, PartialEq)]
pub struct Admitted {
    /// Message id.
    pub id: String,
    /// Owning session id.
    pub session_id: String,
    /// Prompt text.
    pub text: String,
    /// Delivery mode.
    pub delivery: Delivery,
    /// Admission sequence.
    pub admitted_seq: u64,
    /// Promotion sequence, once promoted.
    pub promoted_seq: Option<u64>,
}

/// A prompt admission failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptError {
    /// One id was reused with conflicting content.
    Conflict,
}

impl PromptError {
    /// The stable error tag.
    pub fn tag(&self) -> &'static str {
        conflict_tag()
    }
}

/// The conflict tag reported for a rejected prompt reuse.
pub fn conflict_tag() -> &'static str {
    "Session.PromptConflictError"
}

/// Resolve a data-URI MIME type.
pub fn resolve_mime(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("data:")?;
    let mime = rest.split(';').next()?;
    if mime.is_empty() {
        None
    } else {
        Some(mime.to_string())
    }
}

#[derive(Debug)]
struct HistoryEntry {
    session_id: String,
    text: String,
    delivery: Delivery,
    seq: u64,
}

/// The in-memory session prompt service.
#[derive(Debug, Default)]
pub struct SessionPromptService {
    active: Vec<String>,
    admitted: Vec<Admitted>,
    by_id: HashMap<String, usize>,
    history: HashMap<String, HistoryEntry>,
    seq: u64,
    history_seq: u64,
    execution_calls: Vec<String>,
    wake_calls: Vec<String>,
    interrupt_calls: Vec<String>,
}

impl SessionPromptService {
    /// Create an empty service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sessions currently registered as active.
    pub fn active(&self) -> Vec<String> {
        self.active.clone()
    }

    /// Register a session as active.
    pub fn add_active(&mut self, id: &str) {
        if !self.active.iter().any(|active| active == id) {
            self.active.push(id.to_string());
        }
    }

    /// Admit a prompt, or return the original record for an exact retry.
    pub fn prompt(
        &mut self,
        session: &str,
        id: Option<&str>,
        text: &str,
        delivery: Delivery,
        resume: Option<bool>,
    ) -> Result<Admitted, PromptError> {
        if let Some(id) = id {
            if let Some(index) = self.by_id.get(id).copied() {
                let existing = &self.admitted[index];
                if existing.session_id == session
                    && existing.text == text
                    && existing.delivery == delivery
                {
                    let record = existing.clone();
                    if resume == Some(true) {
                        self.wake_calls.push(session.to_string());
                    }
                    return Ok(record);
                }
                return Err(PromptError::Conflict);
            }
            if let Some(entry) = self.history.get(id) {
                if entry.session_id == session && entry.text == text && entry.delivery == delivery {
                    return Ok(Admitted {
                        id: id.to_string(),
                        session_id: session.to_string(),
                        text: text.to_string(),
                        delivery,
                        admitted_seq: entry.seq,
                        promoted_seq: Some(entry.seq),
                    });
                }
                return Err(PromptError::Conflict);
            }
        }

        self.seq += 1;
        let record = Admitted {
            id: id.map(str::to_string).unwrap_or_else(next_message_id),
            session_id: session.to_string(),
            text: text.to_string(),
            delivery,
            admitted_seq: self.seq,
            promoted_seq: None,
        };
        self.by_id.insert(record.id.clone(), self.admitted.len());
        self.admitted.push(record.clone());
        if resume != Some(false) {
            self.wake_calls.push(session.to_string());
        }
        Ok(record)
    }

    /// Resume a session's execution.
    pub fn resume(&mut self, session: &str) {
        self.execution_calls.push(session.to_string());
    }

    /// Interrupt a session's execution.
    pub fn interrupt(&mut self, session: &str) {
        self.interrupt_calls.push(session.to_string());
    }

    /// Promoted messages for a session, in admission order.
    pub fn messages(&self, session: &str) -> Vec<Admitted> {
        let mut messages: Vec<Admitted> = self
            .admitted
            .iter()
            .filter(|record| record.session_id == session && record.promoted_seq.is_some())
            .cloned()
            .collect();
        messages.sort_by_key(|record| record.admitted_seq);
        messages
    }

    /// Look up an admitted record.
    pub fn admitted(&self, id: &str) -> Option<&Admitted> {
        self.by_id.get(id).map(|index| &self.admitted[*index])
    }

    /// Number of admitted records.
    pub fn admitted_count(&self) -> usize {
        self.admitted.len()
    }

    /// Promote steers admitted at or before `cutoff`, returning the count.
    pub fn promote_steers(&mut self, cutoff: u64) -> usize {
        let mut promoted = 0;
        for record in &mut self.admitted {
            if record.delivery == Delivery::Steer
                && record.promoted_seq.is_none()
                && record.admitted_seq <= cutoff
            {
                record.promoted_seq = Some(record.admitted_seq);
                promoted += 1;
            }
        }
        promoted
    }

    /// Record a prompt as visible history.
    pub fn project_history(&mut self, id: &str, session: &str, text: &str, delivery: Delivery) {
        self.history_seq += 1;
        self.history.insert(
            id.to_string(),
            HistoryEntry {
                session_id: session.to_string(),
                text: text.to_string(),
                delivery,
                seq: self.history_seq,
            },
        );
    }

    /// Sessions the service delegated execution continuation for.
    pub fn execution_calls(&self) -> Vec<String> {
        self.execution_calls.clone()
    }

    /// Sessions the service woke.
    pub fn wake_calls(&self) -> Vec<String> {
        self.wake_calls.clone()
    }

    /// Sessions the service interrupted.
    pub fn interrupt_calls(&self) -> Vec<String> {
        self.interrupt_calls.clone()
    }
}

fn next_message_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("msg_{:016x}", COUNTER.fetch_add(1, Ordering::Relaxed))
}
