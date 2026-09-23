//! Session projection (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/session/projector.ts`: a
//! moved Session projects its new directory, staged/cleared/committed reverts
//! transition the persisted revert state and truncate projected messages at the
//! commit boundary, messages paginate in durable aggregate sequence with
//! next/previous cursors, compaction deltas are not projected while ended
//! compactions project their summary/recent, distinct creator events cannot
//! reuse one projected message ID, and only the newest incomplete assistant is
//! the current assistant projection. The Database/EventV2/SQL wiring and the
//! session-row column assertions are replaced by an in-memory projection.

use serde_json::{json, Value};

/// The persisted revert state.
#[derive(Debug, Clone, PartialEq)]
pub enum RevertState {
    /// No revert in progress.
    None,
    /// A revert is staged.
    Staged {
        /// Boundary message id.
        message_id: String,
        /// Snapshot id, when captured.
        snapshot: Option<String>,
        /// Reverted files.
        files: Vec<String>,
    },
    /// A revert has been committed.
    Committed {
        /// Boundary message id.
        message_id: String,
    },
}

/// A projected message row.
#[derive(Debug, Clone, PartialEq)]
pub struct MsgRow {
    /// Message id.
    pub id: String,
    /// Message kind.
    pub kind: String,
    /// Durable aggregate sequence.
    pub seq: u64,
    /// Message data.
    pub data: Value,
}

/// A projected session.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// Active directory.
    pub directory: String,
    /// Persisted revert state.
    pub revert: RevertState,
    /// Projected messages.
    pub messages: Vec<MsgRow>,
}

impl Projection {
    /// Create an empty projection for a directory.
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
            revert: RevertState::None,
            messages: Vec::new(),
        }
    }

    /// Apply a session-moved event.
    pub fn apply_moved(&mut self, directory: &str) {
        self.directory = directory.to_string();
    }

    /// Apply a staged revert.
    pub fn apply_revert_staged(
        &mut self,
        message_id: &str,
        snapshot: Option<&str>,
        files: Vec<String>,
    ) {
        self.revert = RevertState::Staged {
            message_id: message_id.to_string(),
            snapshot: snapshot.map(str::to_string),
            files,
        };
    }

    /// Apply a cleared revert.
    pub fn apply_revert_cleared(&mut self) {
        self.revert = RevertState::None;
    }

    /// Apply a committed revert, truncating messages after the boundary.
    pub fn apply_revert_committed(&mut self, message_id: &str) {
        self.revert = RevertState::Committed {
            message_id: message_id.to_string(),
        };
        if let Some(index) = self.messages.iter().position(|row| row.id == message_id) {
            self.messages.truncate(index + 1);
        }
    }

    /// Apply a compaction delta, which is not projected.
    pub fn apply_compaction_delta(&mut self, _message_id: &str, _text: &str) {}

    /// Apply an ended compaction, projecting its summary and recent context.
    pub fn apply_compaction_ended(&mut self, message_id: &str, summary: &str, recent: &str) {
        let seq = self.messages.len() as u64 + 1;
        self.messages.push(MsgRow {
            id: message_id.to_string(),
            kind: "compaction".to_string(),
            seq,
            data: json!({ "summary": summary, "recent": recent }),
        });
    }

    /// Insert a creator-event message, rejecting a duplicate id.
    pub fn insert_creator(&mut self, id: &str, kind: &str, data: Value) -> Result<(), String> {
        if self.messages.iter().any(|row| row.id == id) {
            return Err(format!("duplicate projected message id: {id}"));
        }
        let seq = self.messages.len() as u64 + 1;
        self.messages.push(MsgRow {
            id: id.to_string(),
            kind: kind.to_string(),
            seq,
            data,
        });
        Ok(())
    }
}

/// In-memory assistant projection used by the message updater.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantMemory {
    /// Message id.
    pub id: String,
    /// Whether the assistant turn completed.
    pub completed: bool,
}

/// The newest incomplete assistant message index, if any.
pub fn get_current_assistant(messages: &[AssistantMemory]) -> Option<usize> {
    match messages.last() {
        Some(last) if !last.completed => Some(messages.len() - 1),
        _ => None,
    }
}

/// The direction of a pagination cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Page forward.
    Next,
    /// Page backward.
    Previous,
}

/// Paginate message ids around an exclusive cursor.
pub fn paginate(ids: &[String], limit: usize, cursor: Option<(&str, Direction)>) -> Vec<String> {
    match cursor {
        None => ids.iter().take(limit).cloned().collect(),
        Some((cursor, Direction::Next)) => {
            let start = ids
                .iter()
                .position(|id| id == cursor)
                .map(|index| index + 1)
                .unwrap_or(ids.len());
            ids.iter().skip(start).take(limit).cloned().collect()
        }
        Some((cursor, Direction::Previous)) => {
            let end = ids.iter().position(|id| id == cursor).unwrap_or(ids.len());
            let start = end.saturating_sub(limit);
            ids[start..end].to_vec()
        }
    }
}
