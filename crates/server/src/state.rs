//! Shared application state.

use crate::session_store::{InMemorySessionStore, SessionStore};
use opencode_core::InMemoryEventBus;
use std::sync::Arc;

/// State shared by all handlers.
#[derive(Clone)]
pub struct AppState {
    /// The server-wide event bus.
    pub bus: Arc<InMemoryEventBus>,
    /// The session store.
    pub sessions: Arc<dyn SessionStore>,
}

impl AppState {
    /// Create fresh state with an empty event bus and session store.
    pub fn new() -> Self {
        Self {
            bus: Arc::new(InMemoryEventBus::default()),
            sessions: Arc::new(InMemorySessionStore::new()),
        }
    }

    /// Create state with an explicit session store.
    pub fn with_sessions(sessions: Arc<dyn SessionStore>) -> Self {
        Self {
            bus: Arc::new(InMemoryEventBus::default()),
            sessions,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
