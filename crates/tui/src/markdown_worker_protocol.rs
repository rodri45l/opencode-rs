//! Markdown worker protocol reducer.
//!
//! Derived from `packages/session-ui/src/components/markdown-worker-protocol.ts`
//! (upstream 18ef3cc): stable worker tokens accumulate, the unstable tail is
//! replaced, stale responses are ignored, resets bump the generation, only the
//! latest completed state is released, and block keys are namespaced by owner.

use std::fmt;

/// Error raised by the worker protocol.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the worker protocol.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui markdown worker protocol";

/// One `(content, style)` token.
pub type Token = (String, String);

/// Highlight state held by the renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerState {
    pub id: i64,
    pub generation: i64,
    pub language: String,
    pub stable: Vec<Token>,
    pub unstable: Vec<Token>,
}

/// A `highlight` worker response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerResponse {
    pub id: i64,
    pub key: String,
    pub language: String,
    pub reset: bool,
    pub stable: Vec<Token>,
    pub unstable: Vec<Token>,
}

/// Whether the completed response is the latest one for the state.
pub fn should_release_markdown_worker_state(
    released: bool,
    current_id: i64,
    completed_id: i64,
) -> PortResult<bool> {
    Ok(released && current_id == completed_id)
}

/// The namespaced cache key for a block.
pub fn markdown_block_key(
    owner: &str,
    key: Option<&str>,
    index: i64,
    kind: &str,
) -> PortResult<String> {
    Ok(match key {
        Some(key) => format!("{owner}:{key}:{index}:{kind}"),
        None => format!("{owner}:block:{index}"),
    })
}

/// Fold one highlight response into the current state.
pub fn apply_markdown_worker_response(
    current: Option<&WorkerState>,
    response: &WorkerResponse,
) -> PortResult<Option<WorkerState>> {
    if let Some(state) = current {
        if response.id <= state.id {
            return Ok(Some(state.clone()));
        }
    }
    let generation =
        current.map(|state| state.generation).unwrap_or(0) + if response.reset { 1 } else { 0 };
    let mut stable: Vec<Token> = if response.reset {
        response.stable.clone()
    } else {
        current
            .map(|state| state.stable.clone())
            .unwrap_or_default()
    };
    if !response.reset {
        stable.extend(response.stable.iter().cloned());
    }
    Ok(Some(WorkerState {
        id: response.id,
        generation,
        language: response.language.clone(),
        stable,
        unstable: response.unstable.clone(),
    }))
}
