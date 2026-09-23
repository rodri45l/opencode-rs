//! Shared application state.

use opencode_core::InMemoryEventBus;
use std::sync::Arc;

/// State shared by all handlers.
#[derive(Clone)]
pub struct AppState {
    /// The server-wide event bus.
    pub bus: Arc<InMemoryEventBus>,
}

impl AppState {
    /// Create fresh state with an empty event bus.
    pub fn new() -> Self {
        Self {
            bus: Arc::new(InMemoryEventBus::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
