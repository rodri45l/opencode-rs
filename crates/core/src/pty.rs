//! PTY schemas, wire protocol, and websocket tickets.
//!
//! Ports the observable behaviour of `packages/core/src/pty/{schema,protocol,
//! ticket}.ts`: `Info` accepts a non-negative pid and an optional exit code,
//! the protocol drops invalid binary input frames and splits replays into
//! bounded chunks, and tickets are single-use and scoped to a request.

use std::time::Duration;

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
    pub fn decode(_value: &Value) -> CoreResult<Self> {
        Err(CoreError::NotImplemented("pty::Info::decode"))
    }
}

/// The PTY websocket framing helpers.
#[derive(Debug, Default)]
pub struct PtyProtocol;

impl PtyProtocol {
    /// Maximum characters per replay frame.
    pub const REPLAY_CHUNK: usize = 16 * 1024;

    /// Decode an already-decoded text frame.
    pub fn decode_input_str(_input: &str) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "pty::PtyProtocol::decode_input_str",
        ))
    }

    /// Decode a binary input frame, dropping invalid UTF-8.
    pub fn decode_input_bytes(_input: &[u8]) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "pty::PtyProtocol::decode_input_bytes",
        ))
    }

    /// Encode a cursor as a `0x00`-prefixed JSON control frame.
    pub fn meta_frame(_cursor: u64) -> CoreResult<Vec<u8>> {
        Err(CoreError::NotImplemented("pty::PtyProtocol::meta_frame"))
    }

    /// Split a replay payload into bounded frames.
    pub fn chunks(_replay: &str) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented("pty::PtyProtocol::chunks"))
    }
}

/// A PTY identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PtyId(String);

impl PtyId {
    /// Generate a new ascending id.
    pub fn ascending() -> Self {
        Self("pty_00000000000000000000000000".into())
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

/// Single-use, scoped PTY websocket tickets.
#[derive(Debug)]
pub struct PtyTicket {
    ttl: Duration,
}

impl Default for PtyTicket {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30),
        }
    }
}

impl PtyTicket {
    /// Create a ticket store with a custom TTL.
    pub fn new(ttl: Duration) -> Self {
        Self { ttl }
    }

    /// Issue a ticket for `scope`.
    pub fn issue(&self, _scope: &TicketScope) -> CoreResult<IssuedTicket> {
        let _ = self.ttl;
        Err(CoreError::NotImplemented("pty::PtyTicket::issue"))
    }

    /// Consume a ticket, returning whether it was valid for `scope`.
    pub fn consume(&self, _scope: &TicketScope, _ticket: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("pty::PtyTicket::consume"))
    }
}
