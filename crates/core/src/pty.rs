//! PTY schemas, wire protocol, and websocket tickets.
//!
//! Ports the observable behaviour of `packages/core/src/pty/{schema,protocol,
//! ticket}.ts`: `Info` accepts a non-negative pid and an optional exit code,
//! the protocol drops invalid binary input frames and splits replays into
//! bounded chunks, and tickets are single-use and scoped to a request.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Lifecycle status of a PTY session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyStatus {
    /// The process is still running.
    Running,
    /// The process has exited.
    Exited,
}

/// Decoded PTY session information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// Session id (`pty_...`).
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Program being run.
    pub command: String,
    /// Program arguments.
    pub args: Vec<String>,
    /// Working directory.
    pub cwd: String,
    /// Lifecycle status.
    pub status: PtyStatus,
    /// Operating-system process id; `0` when assigned asynchronously.
    pub pid: i64,
    /// Exit code for retained exited sessions.
    pub exit_code: Option<i64>,
}

impl Info {
    /// Decode and validate a PTY info object.
    pub fn decode(value: &Value) -> CoreResult<Self> {
        let string = |field: &str| -> CoreResult<String> {
            value
                .get(field)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| CoreError::Invalid(format!("pty info missing {field}")))
        };
        let id = string("id")?;
        let title = string("title")?;
        let command = string("command")?;
        let cwd = string("cwd")?;
        let args = value
            .get("args")
            .and_then(Value::as_array)
            .map(|args| {
                args.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let pid = value
            .get("pid")
            .and_then(Value::as_i64)
            .ok_or_else(|| CoreError::Invalid("pty info missing pid".into()))?;
        if pid < 0 {
            return Err(CoreError::Invalid("pty pid must be non-negative".into()));
        }
        let status = match value.get("status").and_then(Value::as_str) {
            Some("exited") => PtyStatus::Exited,
            Some("running") => PtyStatus::Running,
            other => {
                return Err(CoreError::Invalid(format!(
                    "invalid pty status: {}",
                    other.unwrap_or("")
                )))
            }
        };
        let exit_code = value.get("exitCode").and_then(Value::as_i64);
        Ok(Self {
            id,
            title,
            command,
            args,
            cwd,
            status,
            pid,
            exit_code,
        })
    }
}

/// The PTY websocket framing helpers.
#[derive(Debug, Default)]
pub struct PtyProtocol;

impl PtyProtocol {
    /// Maximum characters per replay frame.
    pub const REPLAY_CHUNK: usize = 16 * 1024;

    /// Decode an already-decoded text frame.
    pub fn decode_input_str(input: &str) -> CoreResult<Option<String>> {
        Ok(Some(input.to_string()))
    }

    /// Decode a binary input frame, dropping invalid UTF-8.
    pub fn decode_input_bytes(input: &[u8]) -> CoreResult<Option<String>> {
        Ok(String::from_utf8(input.to_vec()).ok())
    }

    /// Encode a cursor as a `0x00`-prefixed JSON control frame.
    pub fn meta_frame(cursor: u64) -> CoreResult<Vec<u8>> {
        let mut frame = vec![0u8];
        let payload = serde_json::to_vec(&serde_json::json!({ "cursor": cursor }))
            .map_err(|error| CoreError::Invalid(error.to_string()))?;
        frame.extend_from_slice(&payload);
        Ok(frame)
    }

    /// Split a replay payload into bounded frames.
    pub fn chunks(replay: &str) -> CoreResult<Vec<String>> {
        if replay.is_empty() {
            return Ok(Vec::new());
        }
        let chars: Vec<char> = replay.chars().collect();
        Ok(chars
            .chunks(Self::REPLAY_CHUNK)
            .map(|chunk| chunk.iter().collect())
            .collect())
    }
}

/// A PTY identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtyId(String);

impl PtyId {
    /// Generate a new ascending id.
    pub fn ascending() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        Self(format!("pty_{millis:013x}{counter:08x}"))
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The scope a ticket authorizes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TicketScope {
    /// PTY id.
    pub pty_id: String,
    /// Directory, when scoped.
    pub directory: Option<String>,
    /// Workspace id, when scoped.
    pub workspace_id: Option<String>,
}

/// An issued ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedTicket {
    /// Opaque ticket value.
    pub ticket: String,
}

#[derive(Debug)]
struct TicketEntry {
    scope: TicketScope,
    issued_at: Instant,
}

/// Single-use, scoped PTY websocket tickets.
#[derive(Debug)]
pub struct PtyTicket {
    ttl: Duration,
    issued: RefCell<HashMap<String, TicketEntry>>,
}

impl Default for PtyTicket {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30),
            issued: RefCell::new(HashMap::new()),
        }
    }
}

impl PtyTicket {
    /// Create a ticket store with a custom TTL.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            issued: RefCell::new(HashMap::new()),
        }
    }

    /// Issue a ticket for `scope`.
    pub fn issue(&self, scope: &TicketScope) -> CoreResult<IssuedTicket> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0);
        let ticket = format!("tkt_{nanos:016x}{counter:08x}");
        self.issued.borrow_mut().insert(
            ticket.clone(),
            TicketEntry {
                scope: scope.clone(),
                issued_at: Instant::now(),
            },
        );
        Ok(IssuedTicket { ticket })
    }

    /// Consume a ticket, returning whether it was valid for `scope`.
    pub fn consume(&self, scope: &TicketScope, ticket: &str) -> CoreResult<bool> {
        let mut issued = self.issued.borrow_mut();
        let Some(entry) = issued.get(ticket) else {
            return Ok(false);
        };
        if entry.issued_at.elapsed() >= self.ttl {
            issued.remove(ticket);
            return Ok(false);
        }
        if entry.scope != *scope {
            return Ok(false);
        }
        issued.remove(ticket);
        Ok(true)
    }
}
