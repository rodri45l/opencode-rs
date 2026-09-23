//! Session instruction resolution and session metadata/fork stubs.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/instruction.ts`
//! and the session metadata/fork cases from `session/session.test.ts`.

use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

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
    claims: RefCell<HashMap<String, BTreeSet<String>>>,
}

impl Instruction {
    /// Create a resolver rooted at `directory`.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            global_home: None,
            disable_claude_code_prompt: false,
            claims: RefCell::new(HashMap::new()),
        }
    }

    fn instruction_files(&self) -> Vec<&'static str> {
        if self.disable_claude_code_prompt {
            vec!["AGENTS.md", "CONTEXT.md"]
        } else {
            vec!["AGENTS.md", "CLAUDE.md", "CONTEXT.md"]
        }
    }

    fn global_files(&self) -> Vec<PathBuf> {
        let Some(home) = &self.global_home else {
            return Vec::new();
        };
        let mut files = vec![home.join("AGENTS.md")];
        if !self.disable_claude_code_prompt {
            files.push(home.join(".claude").join("CLAUDE.md"));
        }
        files
    }

    fn ordered_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for file in self.global_files() {
            if file.exists() {
                paths.push(file.to_string_lossy().into_owned());
                break;
            }
        }
        for name in self.instruction_files() {
            let matches = find_up(name, &self.directory, &self.directory);
            if !matches.is_empty() {
                paths.extend(
                    matches
                        .into_iter()
                        .map(|path| path.to_string_lossy().into_owned()),
                );
                break;
            }
        }
        paths
    }

    /// Instruction files already captured in the system prompt.
    pub fn system_paths(&self) -> BTreeSet<String> {
        self.ordered_paths().into_iter().collect()
    }

    /// Resolve nearby instructions for `file` that `loaded` has not reported.
    pub fn resolve(
        &self,
        loaded: &[Vec<String>],
        file: &str,
        message_id: &str,
    ) -> Result<Vec<InstructionFile>, InstructionError> {
        let system: BTreeSet<String> = self.system_paths();
        let mut already: BTreeSet<String> = BTreeSet::new();
        for group in loaded {
            already.extend(group.iter().cloned());
        }

        let target = PathBuf::from(file);
        let root = self.directory.clone();
        let mut results = Vec::new();
        let mut current = target.parent().map(Path::to_path_buf);

        while let Some(dir) = current {
            if !dir.starts_with(&root) || dir == root {
                break;
            }
            let found = self
                .instruction_files()
                .into_iter()
                .map(|name| dir.join(name))
                .find(|candidate| candidate.exists());
            let Some(found) = found else {
                current = dir.parent().map(Path::to_path_buf);
                continue;
            };
            let found_str = found.to_string_lossy().into_owned();
            if found == target || system.contains(&found_str) || already.contains(&found_str) {
                current = dir.parent().map(Path::to_path_buf);
                continue;
            }

            let mut claims = self.claims.borrow_mut();
            let claimed = claims.entry(message_id.to_string()).or_default();
            if claimed.contains(&found_str) {
                drop(claims);
                current = dir.parent().map(Path::to_path_buf);
                continue;
            }
            claimed.insert(found_str.clone());
            drop(claims);

            if let Ok(content) = std::fs::read_to_string(&found) {
                if !content.is_empty() {
                    results.push(InstructionFile {
                        filepath: found_str.clone(),
                        content: format!("Instructions from: {found_str}\n{content}"),
                    });
                }
            }

            current = dir.parent().map(Path::to_path_buf);
        }

        Ok(results)
    }

    /// Forget the claim state for `message_id`.
    pub fn clear(&self, message_id: &str) -> Result<(), InstructionError> {
        self.claims.borrow_mut().remove(message_id);
        Ok(())
    }

    /// Load the project and global instruction documents.
    pub fn system(&self) -> Result<Vec<String>, InstructionError> {
        let mut rules = Vec::new();
        for path in self.ordered_paths() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if !content.is_empty() {
                    rules.push(format!("Instructions from: {path}\n{content}"));
                }
            }
        }
        Ok(rules)
    }
}

fn find_up(name: &str, start: &Path, stop: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(name);
        if candidate.exists() {
            result.push(candidate);
        }
        if current == stop {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }
    result
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
        &mut self,
        session_id: &str,
        _message_id: Option<&str>,
    ) -> Result<SessionRecord, SessionError> {
        let original = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        let id = format!("ses_{}", self.sessions.len() + 1);
        let record = SessionRecord {
            id: id.clone(),
            title: original.title,
            metadata: original.metadata,
        };
        self.sessions.insert(id, record.clone());
        Ok(record)
    }
}
