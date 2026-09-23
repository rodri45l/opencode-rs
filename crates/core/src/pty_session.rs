//! Pty session helpers.
//!
//! Ports the pure, decidable parts of `packages/core/test/pty/pty-session.test.ts`:
//! typed not-found/exited errors, output-buffer replay with a cursor, exit
//! events carrying the process exit code, and pty creation defaults. The native
//! pty spawn/attach/detach lifecycle and event queue are not reproduced here.

/// Error raised by pty session operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyError {
    /// Placeholder for API compatibility.
    NotImplemented,
    /// The pty session does not exist.
    NotFound(String),
    /// The pty session has already exited.
    Exited(String),
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyError::NotImplemented => write!(f, "not implemented"),
            PtyError::NotFound(id) => write!(f, "pty not found: {id}"),
            PtyError::Exited(id) => write!(f, "pty exited: {id}"),
        }
    }
}

impl std::error::Error for PtyError {}

/// The pty operation being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyOp {
    /// Read a session.
    Get,
    /// Update a session.
    Update,
    /// Remove a session.
    Remove,
    /// Write to a session.
    Write,
    /// Attach to a session.
    Attach,
}

/// A replay of buffered output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay {
    /// The replayed data.
    pub data: String,
    /// The buffer cursor after the replay.
    pub cursor: i64,
}

/// A process exit event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitEvent {
    /// The exit code, if the process exited normally.
    pub exit_code: Option<i32>,
}

/// Defaults for a newly created pty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDefaults {
    /// The command to run.
    pub command: String,
    /// The command arguments.
    pub args: Vec<String>,
    /// The working directory.
    pub cwd: String,
}

/// A session output buffer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutputBuffer {
    data: String,
}

impl OutputBuffer {
    /// Create an empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a chunk to the buffer.
    pub fn append(&mut self, chunk: &str) {
        self.data.push_str(chunk);
    }

    /// Replay the buffer from `cursor` (or the whole buffer when `None`).
    pub fn replay(&self, cursor: Option<i64>) -> Result<Replay, PtyError> {
        let total = self.data.len() as i64;
        match cursor {
            None => Ok(Replay {
                data: self.data.clone(),
                cursor: total,
            }),
            Some(-1) => Ok(Replay {
                data: String::new(),
                cursor: total,
            }),
            Some(offset) => {
                let start = offset.clamp(0, total) as usize;
                Ok(Replay {
                    data: self.data[start..].to_string(),
                    cursor: total,
                })
            }
        }
    }
}

/// Perform `op` against a missing session.
pub fn op_on_missing(_op: PtyOp, id: &str) -> Result<(), PtyError> {
    Err(PtyError::NotFound(id.to_string()))
}

/// Attach to `id`, rejecting a session that has already exited.
pub fn attach_after_exit(id: &str, exited: bool) -> Result<(), PtyError> {
    if exited {
        Err(PtyError::Exited(id.to_string()))
    } else {
        Ok(())
    }
}

/// Build an exit event from an exit code.
pub fn exit_event(exit_code: Option<i32>) -> Result<ExitEvent, PtyError> {
    Ok(ExitEvent { exit_code })
}

/// Defaults for a pty created with `configured_shell` in `cwd`.
pub fn create_defaults(
    configured_shell: Option<&str>,
    cwd: &str,
) -> Result<CreateDefaults, PtyError> {
    let command = configured_shell
        .map(str::to_string)
        .or_else(|| std::env::var("SHELL").ok())
        .unwrap_or_else(|| "/bin/sh".to_string());
    Ok(CreateDefaults {
        command,
        args: vec!["-l".to_string()],
        cwd: cwd.to_string(),
    })
}
