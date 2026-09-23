//! In-memory embedded SDK host.
//!
//! Re-derived from `packages/sdk-next/test/embedded.test.ts` (upstream 18ef3cc):
//! the live SQLite/Effect router is replaced by an in-memory host contract with
//! the same observable behaviour (sessions, model/agent switching, prompt
//! admission, context projection, per-host events, typed not-found errors).

use std::collections::BTreeMap;
use std::fmt;

/// A host-level failure.
#[derive(Debug, PartialEq, Eq)]
pub enum HostError {
    /// The requested session does not exist.
    SessionNotFound,
    /// The requested message does not exist.
    MessageNotFound,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostError::SessionNotFound => f.write_str("SessionNotFoundError"),
            HostError::MessageNotFound => f.write_str("MessageNotFoundError"),
        }
    }
}

impl std::error::Error for HostError {}

/// Result alias for host operations.
pub type PortResult<T> = Result<T, HostError>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "sdk-next embedded host";

/// A model reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRef {
    pub id: String,
    pub provider_id: String,
}

/// A session projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub model: Option<ModelRef>,
    pub agent: Option<String>,
}

/// A context message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMessage {
    pub id: String,
    pub message_type: String,
}

/// The admission returned by a prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptResult {
    pub session_id: String,
    pub message_id: String,
}

/// A recorded host event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub event_type: String,
    pub session_id: Option<String>,
    pub seq: Option<i64>,
}

/// An in-memory host owning its own session state and event log.
pub struct OpenCodeHost {
    sessions: BTreeMap<String, Session>,
    context: BTreeMap<String, Vec<ContextMessage>>,
    events: Vec<Event>,
    next_seq: i64,
    next_message: u64,
}

impl Default for OpenCodeHost {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenCodeHost {
    fn new() -> Self {
        Self {
            sessions: BTreeMap::new(),
            context: BTreeMap::new(),
            events: vec![Event {
                event_type: "server.connected".into(),
                session_id: None,
                seq: None,
            }],
            next_seq: 1,
            next_message: 1,
        }
    }

    fn push_event(&mut self, event_type: &str, session_id: Option<&str>, seq: Option<i64>) {
        self.events.push(Event {
            event_type: event_type.into(),
            session_id: session_id.map(str::to_string),
            seq,
        });
    }

    fn next_message_id(&mut self) -> String {
        let id = format!("msg_{}", self.next_message);
        self.next_message += 1;
        id
    }

    /// Create a session with a generated id.
    pub fn create(&mut self) -> PortResult<Session> {
        let id = format!("ses_{}", self.sessions.len() + 1);
        self.create_with(&id)
    }

    /// Create a session with an explicit id.
    pub fn create_with(&mut self, id: &str) -> PortResult<Session> {
        let session = Session {
            id: id.to_string(),
            model: None,
            agent: None,
        };
        self.sessions.insert(id.to_string(), session.clone());
        self.context.entry(id.to_string()).or_default();
        self.push_event("session.created", Some(id), Some(self.next_seq));
        self.next_seq += 1;
        Ok(session)
    }

    /// Switch the session's model.
    pub fn switch_model(&mut self, session_id: &str, model: ModelRef) -> PortResult<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(HostError::SessionNotFound)?;
        session.model = Some(model);
        let id = self.next_message_id();
        self.context
            .entry(session_id.to_string())
            .or_default()
            .push(ContextMessage {
                id,
                message_type: "model-switched".into(),
            });
        self.push_event(
            "session.next.model.switched",
            Some(session_id),
            Some(self.next_seq),
        );
        self.next_seq += 1;
        Ok(())
    }

    /// Switch the session's agent.
    pub fn switch_agent(&mut self, session_id: &str, agent: &str) -> PortResult<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(HostError::SessionNotFound)?;
        session.agent = Some(agent.to_string());
        self.push_event(
            "session.next.agent.switched",
            Some(session_id),
            Some(self.next_seq),
        );
        self.next_seq += 1;
        Ok(())
    }

    /// Fetch a session.
    pub fn get(&self, session_id: &str) -> PortResult<Session> {
        self.sessions
            .get(session_id)
            .cloned()
            .ok_or(HostError::SessionNotFound)
    }

    /// List all sessions.
    pub fn list(&self) -> PortResult<Vec<Session>> {
        Ok(self.sessions.values().cloned().collect())
    }

    /// The active session set (always empty in the embedded host).
    pub fn active(&self) -> PortResult<BTreeMap<String, String>> {
        Ok(BTreeMap::new())
    }

    /// Admit a prompt for a session.
    pub fn prompt(&mut self, session_id: &str, text: &str) -> PortResult<PromptResult> {
        if !self.sessions.contains_key(session_id) {
            return Err(HostError::SessionNotFound);
        }
        let message_id = self.next_message_id();
        self.context
            .entry(session_id.to_string())
            .or_default()
            .push(ContextMessage {
                id: message_id.clone(),
                message_type: "user".into(),
            });
        let _ = text;
        self.push_event(
            "session.next.prompted",
            Some(session_id),
            Some(self.next_seq),
        );
        self.next_seq += 1;
        Ok(PromptResult {
            session_id: session_id.to_string(),
            message_id,
        })
    }

    /// The context messages for a session.
    pub fn context(&self, session_id: &str) -> PortResult<Vec<ContextMessage>> {
        self.context
            .get(session_id)
            .cloned()
            .ok_or(HostError::SessionNotFound)
    }

    /// Interrupt a session's active run.
    pub fn interrupt(&mut self, session_id: &str) -> PortResult<()> {
        if !self.sessions.contains_key(session_id) {
            return Err(HostError::SessionNotFound);
        }
        Ok(())
    }

    /// Fetch one context message by id.
    pub fn message(&self, session_id: &str, message_id: &str) -> PortResult<ContextMessage> {
        let messages = self
            .context
            .get(session_id)
            .ok_or(HostError::SessionNotFound)?;
        messages
            .iter()
            .find(|message| message.id == message_id)
            .cloned()
            .ok_or(HostError::MessageNotFound)
    }

    /// The host's recorded events.
    pub fn events(&self) -> PortResult<Vec<Event>> {
        Ok(self.events.clone())
    }
}

/// Create a ready embedded host.
pub fn opencode_create() -> PortResult<OpenCodeHost> {
    Ok(OpenCodeHost::new())
}

/// Create the layer-service embedded host.
pub fn opencode_layer_create() -> PortResult<OpenCodeHost> {
    Ok(OpenCodeHost::new())
}
