//! In-memory session storage backing the HTTP read path.
//!
//! The reference persists sessions and messages in SQLite; the Rust port keeps
//! an equivalent observable model in memory until the storage phase lands.
//! Ordering, filtering, search and cursor semantics follow
//! `packages/opencode/src/session/session.ts` and `session/message-v2.ts`.

use crate::session::{
    GlobalSessionInfo, ProjectInfo, SessionHistory, SessionInfo, SessionTime, TokenCache, Tokens,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use opencode_protocol::Order;
use opencode_schema::SessionId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Input accepted by [`SessionStore::create`].
#[derive(Debug, Clone, Default)]
pub struct CreateSession {
    /// Session title.
    pub title: Option<String>,
    /// Free-form metadata.
    pub metadata: Option<Value>,
    /// Parent session id, when forking/creating a child.
    pub parent_id: Option<String>,
    /// Instance directory.
    pub directory: Option<String>,
    /// Default agent id.
    pub agent: Option<String>,
    /// Default model reference.
    pub model: Option<Value>,
}

/// Filters accepted by the legacy session list route.
#[derive(Debug, Clone, Default)]
pub struct SessionListQuery {
    /// Restrict to a directory.
    pub directory: Option<String>,
    /// Restrict to root sessions (no parent).
    pub roots: Option<bool>,
    /// Lower bound on creation time.
    pub start: Option<i64>,
    /// Case-insensitive title search.
    pub search: Option<String>,
    /// Maximum number of items.
    pub limit: Option<usize>,
    /// Sort order (defaults to newest first).
    pub order: Option<Order>,
}

/// Filters accepted by the experimental/global session list route.
#[derive(Debug, Clone, Default)]
pub struct GlobalListQuery {
    /// Restrict to a directory.
    pub directory: Option<String>,
    /// Restrict to root sessions (no parent).
    pub roots: Option<bool>,
    /// Lower bound on creation time.
    pub start: Option<i64>,
    /// Case-insensitive title search.
    pub search: Option<String>,
    /// Maximum number of items.
    pub limit: Option<usize>,
    /// Sort order (defaults to newest first).
    pub order: Option<Order>,
    /// Include archived sessions.
    pub archived: bool,
    /// Exclusive upper bound on the update time.
    pub cursor: Option<i64>,
}

/// Fields mutable through the session update route.
#[derive(Debug, Clone, Default)]
pub struct SessionPatch {
    /// Replacement title.
    pub title: Option<String>,
    /// Replacement metadata.
    pub metadata: Option<Value>,
    /// Replacement permission ruleset.
    pub permission: Option<Value>,
    /// Archive timestamp.
    pub archived: Option<i64>,
}

/// One page of session messages.
#[derive(Debug, Clone, PartialEq)]
pub struct MessagePage {
    /// Messages in ascending chronological order.
    pub items: Vec<Value>,
    /// Cursor pointing at the oldest item of the page, when older items remain.
    pub cursor: Option<String>,
}

/// Failure modes for a message page request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagePageError {
    /// The session does not exist.
    SessionNotFound,
    /// The supplied cursor could not be decoded.
    InvalidCursor,
}

/// Storage operations required by the session HTTP routes.
pub trait SessionStore: Send + Sync + 'static {
    /// Create a session.
    fn create(&self, input: CreateSession) -> SessionInfo;

    /// Look up a session by id.
    fn get(&self, id: &str) -> Option<SessionInfo>;

    /// List sessions honouring `query`.
    fn list(&self, query: &SessionListQuery) -> Vec<SessionInfo>;

    /// List global sessions (with project metadata) honouring `query`.
    fn list_global(&self, query: &GlobalListQuery) -> Vec<GlobalSessionInfo>;

    /// Apply a patch, returning the updated session.
    fn update(&self, id: &str, patch: SessionPatch) -> Option<SessionInfo>;

    /// Remove a session and its messages.
    fn remove(&self, id: &str) -> bool;

    /// Fork a session, copying metadata and the chronological message prefix.
    fn fork(&self, id: &str, message_id: Option<&str>) -> Option<SessionInfo>;

    /// Read one page of messages.
    fn messages(
        &self,
        session_id: &str,
        limit: Option<usize>,
        before: Option<&str>,
    ) -> Result<MessagePage, MessagePageError>;

    /// Read a single message with its parts.
    fn message(&self, session_id: &str, message_id: &str) -> Option<Value>;

    /// Read a page of durable session events.
    fn history(&self, session_id: &str) -> Option<SessionHistory>;

    /// Read the active context messages.
    fn context(&self, session_id: &str) -> Option<Vec<Value>>;
}

#[derive(Default)]
struct Inner {
    sessions: HashMap<String, SessionInfo>,
    order: Vec<String>,
    messages: HashMap<String, Vec<StoredMessage>>,
    clock: i64,
}

#[derive(Clone)]
struct StoredMessage {
    info: Value,
    parts: Vec<Value>,
    created: i64,
    id: String,
}

/// An in-memory [`SessionStore`].
pub struct InMemorySessionStore {
    inner: Mutex<Inner>,
}

impl InMemorySessionStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn project_id(directory: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in directory.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("prj_{hash:016x}")
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "session".to_string()
    } else {
        slug
    }
}

fn zero_tokens() -> Tokens {
    Tokens {
        input: 0,
        output: 0,
        reasoning: 0,
        cache: TokenCache { read: 0, write: 0 },
    }
}

fn updated_key(info: &SessionInfo) -> i64 {
    info.time.updated.unwrap_or(info.time.created)
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageCursor {
    id: String,
    time: i64,
}

fn encode_message_cursor(id: &str, time: i64) -> String {
    let cursor = MessageCursor {
        id: id.to_string(),
        time,
    };
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&cursor).unwrap_or_default())
}

fn decode_message_cursor(value: &str) -> Option<MessageCursor> {
    let bytes = URL_SAFE_NO_PAD.decode(value).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn message_value(message: &StoredMessage) -> Value {
    json!({ "info": message.info, "parts": message.parts })
}

impl Inner {
    fn bump(&mut self) -> i64 {
        let now = now_millis();
        let next = if now > self.clock {
            now
        } else {
            self.clock + 1
        };
        self.clock = next;
        next
    }
}

impl SessionStore for InMemorySessionStore {
    fn create(&self, input: CreateSession) -> SessionInfo {
        let mut inner = self.inner.lock().expect("session store poisoned");
        let created = inner.bump();
        let directory = input.directory.unwrap_or_else(|| ".".to_string());
        let title = input.title.unwrap_or_else(|| "New session".to_string());
        let info = SessionInfo {
            id: SessionId::generate().to_string(),
            slug: slugify(&title),
            project_id: project_id(&directory),
            workspace_id: None,
            directory,
            parent_id: input.parent_id,
            summary: None,
            cost: 0.0,
            tokens: zero_tokens(),
            share: None,
            title,
            version: "0.0.0".to_string(),
            metadata: input.metadata,
            time: SessionTime {
                created,
                updated: Some(created),
                compacting: None,
                archived: None,
            },
            permission: None,
            revert: None,
        };
        inner.sessions.insert(info.id.clone(), info.clone());
        inner.order.push(info.id.clone());
        info
    }

    fn get(&self, id: &str) -> Option<SessionInfo> {
        let inner = self.inner.lock().expect("session store poisoned");
        inner.sessions.get(id).cloned()
    }

    fn list(&self, query: &SessionListQuery) -> Vec<SessionInfo> {
        let inner = self.inner.lock().expect("session store poisoned");
        let mut items: Vec<SessionInfo> = inner.sessions.values().cloned().collect();
        if let Some(directory) = &query.directory {
            items.retain(|info| &info.directory == directory);
        }
        if query.roots == Some(true) {
            items.retain(|info| info.parent_id.is_none());
        }
        if let Some(start) = query.start {
            items.retain(|info| info.time.created >= start);
        }
        if let Some(search) = &query.search {
            let needle = search.to_lowercase();
            items.retain(|info| info.title.to_lowercase().contains(&needle));
        }
        sort_sessions(&mut items, query.order);
        if let Some(limit) = query.limit {
            items.truncate(limit);
        }
        items
    }

    fn list_global(&self, query: &GlobalListQuery) -> Vec<GlobalSessionInfo> {
        let inner = self.inner.lock().expect("session store poisoned");
        let mut items: Vec<SessionInfo> = inner.sessions.values().cloned().collect();
        if let Some(directory) = &query.directory {
            items.retain(|info| &info.directory == directory);
        }
        if query.roots == Some(true) {
            items.retain(|info| info.parent_id.is_none());
        }
        if let Some(start) = query.start {
            items.retain(|info| info.time.created >= start);
        }
        if !query.archived {
            items.retain(|info| info.time.archived.is_none());
        }
        if let Some(cursor) = query.cursor {
            items.retain(|info| updated_key(info) < cursor);
        }
        if let Some(search) = &query.search {
            let needle = search.to_lowercase();
            items.retain(|info| info.title.to_lowercase().contains(&needle));
        }
        sort_sessions(&mut items, query.order);
        if let Some(limit) = query.limit {
            items.truncate(limit);
        }
        items
            .into_iter()
            .map(|info| {
                let project = ProjectInfo {
                    id: info.project_id.clone(),
                    name: None,
                    worktree: info.directory.clone(),
                };
                GlobalSessionInfo {
                    info,
                    project: Some(project),
                }
            })
            .collect()
    }

    fn update(&self, id: &str, patch: SessionPatch) -> Option<SessionInfo> {
        let mut inner = self.inner.lock().expect("session store poisoned");
        if !inner.sessions.contains_key(id) {
            return None;
        }
        let updated = inner.bump();
        let session = inner.sessions.get_mut(id)?;
        if let Some(title) = patch.title {
            session.title = title;
        }
        if let Some(metadata) = patch.metadata {
            session.metadata = Some(metadata);
        }
        if let Some(permission) = patch.permission {
            session.permission = Some(permission);
        }
        if let Some(archived) = patch.archived {
            session.time.archived = Some(archived);
        }
        session.time.updated = Some(updated);
        Some(session.clone())
    }

    fn remove(&self, id: &str) -> bool {
        let mut inner = self.inner.lock().expect("session store poisoned");
        if inner.sessions.remove(id).is_none() {
            return false;
        }
        inner.order.retain(|existing| existing != id);
        inner.messages.remove(id);
        true
    }

    fn fork(&self, id: &str, _message_id: Option<&str>) -> Option<SessionInfo> {
        let mut inner = self.inner.lock().expect("session store poisoned");
        let original = inner.sessions.get(id).cloned()?;
        let created = inner.bump();
        let parent_id = original.id.clone();
        let info = SessionInfo {
            id: SessionId::generate().to_string(),
            parent_id: Some(parent_id),
            time: SessionTime {
                created,
                updated: Some(created),
                compacting: None,
                archived: None,
            },
            ..original
        };
        inner.sessions.insert(info.id.clone(), info.clone());
        inner.order.push(info.id.clone());
        Some(info)
    }

    fn messages(
        &self,
        session_id: &str,
        limit: Option<usize>,
        before: Option<&str>,
    ) -> Result<MessagePage, MessagePageError> {
        let inner = self.inner.lock().expect("session store poisoned");
        let decoded_before = match before {
            Some(before) => {
                Some(decode_message_cursor(before).ok_or(MessagePageError::InvalidCursor)?)
            }
            None => None,
        };
        if !inner.sessions.contains_key(session_id) {
            return Err(MessagePageError::SessionNotFound);
        }
        let mut list = inner.messages.get(session_id).cloned().unwrap_or_default();
        list.sort_by_key(|message| (message.created, message.id.clone()));
        if let Some(cursor) = decoded_before {
            list.retain(|message| {
                message.created < cursor.time
                    || (message.created == cursor.time && message.id < cursor.id)
            });
        }
        let limit = limit.unwrap_or(0);
        if limit == 0 {
            return Ok(MessagePage {
                items: list.iter().map(message_value).collect(),
                cursor: None,
            });
        }
        let more = list.len() > limit;
        let page = if more {
            list.split_off(list.len() - limit)
        } else {
            list
        };
        let cursor = if more {
            page.first()
                .map(|message| encode_message_cursor(&message.id, message.created))
        } else {
            None
        };
        Ok(MessagePage {
            items: page.iter().map(message_value).collect(),
            cursor,
        })
    }

    fn message(&self, session_id: &str, message_id: &str) -> Option<Value> {
        let inner = self.inner.lock().expect("session store poisoned");
        inner
            .messages
            .get(session_id)
            .and_then(|list| list.iter().find(|message| message.id == message_id))
            .map(message_value)
    }

    fn history(&self, session_id: &str) -> Option<SessionHistory> {
        let inner = self.inner.lock().expect("session store poisoned");
        inner
            .sessions
            .contains_key(session_id)
            .then(|| SessionHistory {
                data: Vec::new(),
                has_more: false,
            })
    }

    fn context(&self, session_id: &str) -> Option<Vec<Value>> {
        let inner = self.inner.lock().expect("session store poisoned");
        inner.sessions.contains_key(session_id).then(Vec::new)
    }
}

fn sort_sessions(items: &mut [SessionInfo], order: Option<Order>) {
    items.sort_by_key(|info| (updated_key(info), info.time.created, info.id.clone()));
    if !matches!(order, Some(Order::Asc)) {
        items.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(store: &InMemorySessionStore, title: &str, directory: &str) -> SessionInfo {
        store.create(CreateSession {
            title: Some(title.to_string()),
            directory: Some(directory.to_string()),
            ..CreateSession::default()
        })
    }

    #[test]
    fn filters_and_orders_sessions() {
        let store = InMemorySessionStore::new();
        let root = create(&store, "root-session", "/tmp/a");
        let child = store.create(CreateSession {
            title: Some("child-session".to_string()),
            parent_id: Some(root.id.clone()),
            directory: Some("/tmp/a".to_string()),
            ..CreateSession::default()
        });

        let roots = store.list(&SessionListQuery {
            roots: Some(true),
            ..SessionListQuery::default()
        });
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, root.id);
        assert!(store
            .list(&SessionListQuery {
                search: Some("child".to_string()),
                ..SessionListQuery::default()
            })
            .iter()
            .any(|info| info.id == child.id));
    }

    #[test]
    fn paginates_messages_with_cursor() {
        let store = InMemorySessionStore::new();
        let session = create(&store, "messages", "/tmp/a");
        for index in 1..=5 {
            let id = format!("msg_{index}");
            let inner = &mut *store.inner.lock().unwrap();
            inner
                .messages
                .entry(session.id.clone())
                .or_default()
                .push(StoredMessage {
                    info: json!({ "id": id, "sessionID": session.id, "role": "user" }),
                    parts: Vec::new(),
                    created: index,
                    id,
                });
        }

        let first = store.messages(&session.id, Some(2), None).unwrap();
        assert_eq!(first.items.len(), 2);
        assert!(first.cursor.is_some());
        let second = store
            .messages(&session.id, Some(2), first.cursor.as_deref())
            .unwrap();
        assert_eq!(second.items.len(), 2);
        assert_ne!(first.items[0], second.items[0]);
    }

    #[test]
    fn missing_session_reports_not_found() {
        let store = InMemorySessionStore::new();
        assert_eq!(
            store.messages("ses_missing", Some(2), None),
            Err(MessagePageError::SessionNotFound)
        );
        assert_eq!(
            store.messages("ses_missing", Some(2), Some("bad")),
            Err(MessagePageError::InvalidCursor)
        );
    }
}
