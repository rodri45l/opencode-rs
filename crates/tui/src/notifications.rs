//! Internal attention notifications reducer.
//!
//! Port of packages/tui/src/feature-plugins/system/notifications.ts behaviour
//! (upstream 18ef3cc).

use std::collections::{HashMap, HashSet};

/// A known session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub parent_id: Option<String>,
}

/// A session status transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Busy,
    Retry,
    Idle,
}

/// A sound to play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundConfig {
    pub name: String,
    pub when: String,
}

/// An attention notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyInput {
    pub title: Option<String>,
    pub message: String,
    pub notification: Option<String>,
    pub sound: SoundConfig,
}

/// An event the notifier reacts to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationEvent {
    QuestionAsked {
        id: String,
        session_id: String,
    },
    QuestionReplied {
        request_id: String,
    },
    QuestionRejected {
        request_id: String,
    },
    PermissionAsked {
        id: String,
        session_id: String,
    },
    PermissionReplied {
        request_id: String,
    },
    SessionStatus {
        session_id: String,
        status: SessionStatus,
    },
    SessionError {
        session_id: Option<String>,
        error_name: String,
        error_message: String,
    },
}

/// The notification reducer state.
pub struct Notifier {
    sessions: HashMap<String, SessionInfo>,
    active: HashSet<String>,
    errored: HashSet<String>,
    questions: HashSet<String>,
    permissions: HashSet<String>,
}

impl Notifier {
    /// Create a notifier over the known sessions.
    pub fn new(sessions: Vec<SessionInfo>) -> Self {
        Notifier {
            sessions: sessions
                .into_iter()
                .map(|session| (session.id.clone(), session))
                .collect(),
            active: HashSet::new(),
            errored: HashSet::new(),
            questions: HashSet::new(),
            permissions: HashSet::new(),
        }
    }

    /// Apply an event, returning a notification when one is emitted.
    pub fn emit(&mut self, event: NotificationEvent) -> Option<NotifyInput> {
        match event {
            NotificationEvent::QuestionAsked { id, session_id } => {
                if !self.questions.insert(id) {
                    return None;
                }
                Some(self.notify(Some(&session_id), "Question needs input", "question"))
            }
            NotificationEvent::QuestionReplied { request_id }
            | NotificationEvent::QuestionRejected { request_id } => {
                self.questions.remove(&request_id);
                None
            }
            NotificationEvent::PermissionAsked { id, session_id } => {
                if !self.permissions.insert(id) {
                    return None;
                }
                Some(self.notify(Some(&session_id), "Permission needs input", "permission"))
            }
            NotificationEvent::PermissionReplied { request_id } => {
                self.permissions.remove(&request_id);
                None
            }
            NotificationEvent::SessionStatus { session_id, status } => {
                if matches!(status, SessionStatus::Busy | SessionStatus::Retry) {
                    self.active.insert(session_id.clone());
                    self.errored.remove(&session_id);
                    return None;
                }
                if !self.active.remove(&session_id) {
                    return None;
                }
                if self.errored.remove(&session_id) {
                    return None;
                }
                let sound = if self
                    .sessions
                    .get(&session_id)
                    .and_then(|session| session.parent_id.as_ref())
                    .is_some()
                {
                    "subagent_done"
                } else {
                    "done"
                };
                Some(self.notify(Some(&session_id), "Session done", sound))
            }
            NotificationEvent::SessionError {
                session_id,
                error_name,
                error_message,
            } => {
                let session_id = session_id?;
                if !self.active.contains(&session_id) {
                    return None;
                }
                self.errored.insert(session_id.clone());
                Some(self.notify(
                    Some(&session_id),
                    session_error_message(&error_name, &error_message),
                    "error",
                ))
            }
        }
    }

    fn notify(&self, session_id: Option<&str>, message: &str, sound: &str) -> NotifyInput {
        let session = session_id.and_then(|id| self.sessions.get(id));
        let is_subagent = session
            .and_then(|session| session.parent_id.as_ref())
            .is_some();
        NotifyInput {
            title: session.map(|session| session.title.clone()),
            message: message.to_string(),
            notification: if is_subagent {
                None
            } else {
                Some("blurred".to_string())
            },
            sound: SoundConfig {
                name: sound.to_string(),
                when: "always".to_string(),
            },
        }
    }
}

fn session_error_message(error_name: &str, data_message: &str) -> &'static str {
    if error_name == "MessageAbortedError" {
        return "Session aborted";
    }
    if data_message == "SSE read timed out" {
        return "Model stopped responding";
    }
    "Session error"
}
