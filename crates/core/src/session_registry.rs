//! Session create registry (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/session.ts#create`: an
//! omitted ID creates a fresh projected Session, an exact ID retry returns the
//! original record and ignores differing create arguments, supplied
//! location/agent/model attributes are stored, a reused ID returns the current
//! projection after updates, agent and model switches publish durable Session
//! events (an unchanged or default-variant model switch publishes nothing), a
//! missing Session reports `Session.NotFoundError`, and unfinished shell/skill
//! operations report their operation name. The Database/EventV2/projector and
//! Effect layers are replaced by an in-memory registry with an event log.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};

/// A session record.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// Session id.
    pub id: String,
    /// Project id.
    pub project_id: String,
    /// Active directory.
    pub directory: String,
    /// Workspace id.
    pub workspace_id: Option<String>,
    /// Selected agent.
    pub agent: Option<String>,
    /// Selected model.
    pub model: Option<Value>,
}

/// A typed session-create failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateError {
    /// The session does not exist.
    NotFound,
    /// The operation is not available yet.
    OperationUnavailable {
        /// Operation name.
        operation: String,
    },
}

impl CreateError {
    /// The stable error tag.
    pub fn tag(&self) -> &'static str {
        match self {
            CreateError::NotFound => "Session.NotFoundError",
            CreateError::OperationUnavailable { .. } => "Session.OperationUnavailableError",
        }
    }

    /// The unavailable operation name, if any.
    pub fn operation(&self) -> Option<&str> {
        match self {
            CreateError::OperationUnavailable { operation } => Some(operation.as_str()),
            CreateError::NotFound => None,
        }
    }
}

/// An in-memory session registry with an event log.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    sessions: Vec<Session>,
    events: Vec<Value>,
}

impl SessionRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create (or return) a session by id.
    pub fn create(
        &mut self,
        id: Option<&str>,
        directory: &str,
        workspace_id: Option<&str>,
        agent: Option<&str>,
        model: Option<Value>,
    ) -> Session {
        if let Some(id) = id {
            if let Some(existing) = self.sessions.iter().find(|session| session.id == id) {
                return existing.clone();
            }
        }
        let id = match id {
            Some(id) => id.to_string(),
            None => next_session_id(),
        };
        let session = Session {
            id: id.clone(),
            project_id: "project".to_string(),
            directory: directory.to_string(),
            workspace_id: workspace_id.map(str::to_string),
            agent: agent.map(str::to_string),
            model,
        };
        self.sessions.push(session.clone());
        self.events
            .push(json!({ "type": "session.created", "data": { "id": id } }));
        session
    }

    /// All sessions in creation order.
    pub fn list(&self) -> Vec<Session> {
        self.sessions.clone()
    }

    /// Look up a session.
    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id == id)
    }

    /// Update the selected agent without publishing an event.
    pub fn update_agent(&mut self, id: &str, agent: &str) -> Result<(), CreateError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|session| session.id == id)
            .ok_or(CreateError::NotFound)?;
        session.agent = Some(agent.to_string());
        Ok(())
    }

    /// Switch the selected agent through a durable session event.
    pub fn switch_agent(&mut self, id: &str, agent: &str) -> Result<(), CreateError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|session| session.id == id)
            .ok_or(CreateError::NotFound)?;
        session.agent = Some(agent.to_string());
        self.events.push(json!({
            "type": "session.next.agent.switched",
            "data": { "agent": agent },
        }));
        Ok(())
    }

    /// Switch the selected model through a durable session event.
    pub fn switch_model(&mut self, id: &str, model: &Value) -> Result<(), CreateError> {
        let session = self
            .sessions
            .iter_mut()
            .find(|session| session.id == id)
            .ok_or(CreateError::NotFound)?;
        if session
            .model
            .as_ref()
            .map(|current| normalize_model(current) == normalize_model(model))
            .unwrap_or(false)
        {
            return Ok(());
        }
        session.model = Some(model.clone());
        self.events.push(json!({
            "type": "session.next.model.switched",
            "data": { "model": model },
        }));
        Ok(())
    }

    /// Run a shell operation, which is not available yet.
    pub fn shell(&self, _id: &str) -> Result<(), CreateError> {
        Err(CreateError::OperationUnavailable {
            operation: "shell".to_string(),
        })
    }

    /// Run a skill operation, which is not available yet.
    pub fn skill(&self, _id: &str) -> Result<(), CreateError> {
        Err(CreateError::OperationUnavailable {
            operation: "skill".to_string(),
        })
    }

    /// Number of published events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// The most recently published event.
    pub fn last_event(&self) -> Option<&Value> {
        self.events.last()
    }
}

fn normalize_model(model: &Value) -> Value {
    let mut normalized = model.clone();
    let variant = normalized.get("variant").and_then(Value::as_str);
    if variant.is_none() || variant == Some("default") {
        normalized["variant"] = json!("default");
    }
    normalized
}

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_session_id() -> String {
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("ses_{counter:016x}")
}
