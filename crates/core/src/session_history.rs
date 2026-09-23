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
    pub fn create(&mut self, _session_id: &str) -> CoreResult<()> {
        let _ = &self.events;
        Err(CoreError::NotImplemented(
            "session_history::SessionHistory::create",
        ))
    }

    /// Append a public event, returning its aggregate sequence.
    pub fn append(&mut self, _session_id: &str, _event_type: &str) -> CoreResult<u64> {
        let _ = &self.events;
        Err(CoreError::NotImplemented(
            "session_history::SessionHistory::append",
        ))
    }

    /// Page history after an exclusive cursor.
    pub fn history(
        &self,
        _session_id: &str,
        _after: Option<u64>,
        _limit: usize,
    ) -> CoreResult<HistoryPage> {
        let _ = &self.events;
        Err(CoreError::NotImplemented(
            "session_history::SessionHistory::history",
        ))
    }
}
