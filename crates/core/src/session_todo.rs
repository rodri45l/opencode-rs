//! Session todo storage (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/session/todo.ts`:
//! `update` replaces the persisted todos for a session in the given order and
//! publishes an `updated` payload `{ sessionID, todos }`, and `get` returns the
//! current list (empty after an empty update). The `Database`/`TodoTable`/event
//! service wiring is replaced by an in-memory store.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::{CoreError, CoreResult};

/// One persisted todo entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    /// The todo content.
    pub content: String,
    /// The status.
    pub status: String,
    /// The priority.
    pub priority: String,
}

impl Todo {
    /// The JSON representation used in published payloads.
    pub fn to_json(&self) -> Value {
        json!({
            "content": self.content,
            "status": self.status,
            "priority": self.priority,
        })
    }
}

/// In-memory session todo storage.
#[derive(Debug, Default)]
pub struct SessionTodoStore {
    todos: BTreeMap<String, Vec<Todo>>,
    published: Vec<Value>,
}

impl SessionTodoStore {
    /// Create empty storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the todos for `session_id` in order and publish an update.
    pub fn update(&mut self, _session_id: &str, _todos: Vec<Todo>) -> CoreResult<()> {
        let _ = (&mut self.todos, &mut self.published);
        Err(CoreError::NotImplemented(
            "session_todo::SessionTodoStore::update",
        ))
    }

    /// The current todos for `session_id`.
    pub fn get(&self, _session_id: &str) -> CoreResult<Vec<Todo>> {
        let _ = &self.todos;
        Err(CoreError::NotImplemented(
            "session_todo::SessionTodoStore::get",
        ))
    }

    /// The published `updated` payloads in order.
    pub fn published(&self) -> CoreResult<Vec<Value>> {
        let _ = &self.published;
        Err(CoreError::NotImplemented(
            "session_todo::SessionTodoStore::published",
        ))
    }
}
