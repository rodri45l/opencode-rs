//! Session event history pagination.
//!
//! Ports the observable behaviour of `packages/core/src/session.ts#history`:
//! history pages are returned in aggregate sequence order, `after` is an
//! exclusive cursor, `hasMore` reports exhaustion, and a missing session fails
//! with a not-found error.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// A public history event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEvent {
    /// Aggregate sequence number.
    pub seq: u64,
    /// Versioned event type.
    pub event_type: String,
}

/// A page of session history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPage {
    /// Events in aggregate order.
    pub events: Vec<HistoryEvent>,
    /// Whether more events remain.
    pub has_more: bool,
}

/// Session history storage.
#[derive(Debug, Default)]
pub struct SessionHistory {
    events: BTreeMap<String, Vec<HistoryEvent>>,
}

impl SessionHistory {
    /// Create empty history storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a session.
    pub fn create(&mut self, session_id: &str) -> CoreResult<()> {
        self.events.entry(session_id.to_string()).or_default();
        Ok(())
    }

    /// Append a public event, returning its aggregate sequence.
    pub fn append(&mut self, session_id: &str, event_type: &str) -> CoreResult<u64> {
        let events = self
            .events
            .get_mut(session_id)
            .ok_or_else(|| not_found(session_id))?;
        let seq = events.len() as u64 + 1;
        events.push(HistoryEvent {
            seq,
            event_type: event_type.to_string(),
        });
        Ok(seq)
    }

    /// Page history after an exclusive cursor.
    pub fn history(
        &self,
        session_id: &str,
        after: Option<u64>,
        limit: usize,
    ) -> CoreResult<HistoryPage> {
        let events = self
            .events
            .get(session_id)
            .ok_or_else(|| not_found(session_id))?;
        let cursor = after.unwrap_or(0);
        let remaining: Vec<HistoryEvent> = events
            .iter()
            .filter(|event| event.seq > cursor)
            .cloned()
            .collect();
        let has_more = remaining.len() > limit;
        let page = remaining.into_iter().take(limit).collect();
        Ok(HistoryPage {
            events: page,
            has_more,
        })
    }
}

fn not_found(session_id: &str) -> CoreError {
    CoreError::Invalid(format!("session not found: {session_id}"))
}
