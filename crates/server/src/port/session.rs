//! Session instruction resolution and session metadata/fork stubs.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/instruction.ts`
//! and the session metadata/fork cases from `session/session.test.ts`.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A resolved instruction document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionFile {
    /// Absolute path of the instruction file.
    pub filepath: String,
    /// File contents.
    pub content: String,
}

/// A typed instruction-resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for InstructionError {}

/// Resolves `AGENTS.md`/`CLAUDE.md` instructions for a session.
#[derive(Debug, Clone)]
pub struct Instruction {
    /// Project directory.
    pub directory: PathBuf,
    /// Global home directory, when configured.
    pub global_home: Option<PathBuf>,
    /// Skip Claude Code prompt files.
    pub disable_claude_code_prompt: bool,
}

impl Instruction {
    /// Create a resolver rooted at `directory`.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            global_home: None,
            disable_claude_code_prompt: false,
        }
    }

    /// Instruction files already captured in the system prompt.
    pub fn system_paths(&self) -> BTreeSet<String> {
        BTreeSet::new()
    }

    /// Resolve nearby instructions for `file` that `loaded` has not reported.
    pub fn resolve(
        &self,
        _loaded: &[Vec<String>],
        _file: &str,
        _message_id: &str,
    ) -> Result<Vec<InstructionFile>, InstructionError> {
        Err(InstructionError::NotImplemented("instruction::resolve"))
    }

    /// Forget the claim state for `message_id`.
    pub fn clear(&self, _message_id: &str) -> Result<(), InstructionError> {
        Err(InstructionError::NotImplemented("instruction::clear"))
    }

    /// Load the project and global instruction documents.
    pub fn system(&self) -> Result<Vec<String>, InstructionError> {
        Err(InstructionError::NotImplemented("instruction::system"))
    }
}

/// A stored session record.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionRecord {
    /// Session id.
    pub id: String,
    /// Session title.
    pub title: String,
    /// Free-form metadata, omitted when unset.
    pub metadata: Option<Value>,
}

/// A stored message reference used for fork ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRef {
    /// Message id.
    pub id: String,
    /// Creation timestamp used for chronological ordering.
    pub created: i64,
}

/// A typed session-store failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
    /// The session does not exist.
    NotFound(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::NotFound(id) => write!(f, "session not found: {id}"),
        }
    }
}

impl std::error::Error for SessionError {}

/// An in-memory session store.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionStore {
    sessions: std::collections::HashMap<String, SessionRecord>,
    messages: std::collections::HashMap<String, Vec<MessageRef>>,
}

impl SessionStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a session with an optional metadata value.
    pub fn create(&mut self, title: &str, metadata: Option<Value>) -> SessionRecord {
        let id = format!("ses_{}", self.sessions.len() + 1);
        let record = SessionRecord {
            id: id.clone(),
            title: title.to_string(),
            metadata,
        };
        self.sessions.insert(id, record.clone());
        record
    }

    /// Look up a session.
    pub fn get(&self, id: &str) -> Option<SessionRecord> {
        self.sessions.get(id).cloned()
    }

    /// Record a message under a session.
    pub fn update_message(&mut self, session_id: &str, id: &str, created: i64) {
        self.messages
            .entry(session_id.to_string())
            .or_default()
            .push(MessageRef {
                id: id.to_string(),
                created,
            });
    }

    /// The chronological message prefix copied into a fork.
    pub fn fork_prefix(&self, session_id: &str, message_id: Option<&str>) -> Vec<MessageRef> {
        let messages = self.messages.get(session_id).cloned().unwrap_or_default();
        let Some(cutoff) = message_id
            .and_then(|id| messages.iter().find(|message| message.id == id))
            .map(|message| message.created)
        else {
            return messages;
        };
        messages
            .into_iter()
            .filter(|message| message.created < cutoff)
            .collect()
    }

    /// Fork a session, copying metadata and the chronological message prefix.
    pub fn fork(
        &self,
        _session_id: &str,
        _message_id: Option<&str>,
    ) -> Result<SessionRecord, SessionError> {
        Err(SessionError::NotImplemented("session::fork"))
    }
}
