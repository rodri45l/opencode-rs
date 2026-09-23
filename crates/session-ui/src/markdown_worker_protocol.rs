//! Markdown highlight worker protocol state.
//!
//! Port of packages/session-ui/src/components/markdown-worker-protocol.ts
//! behaviour (upstream 18ef3cc).

/// A highlighted token: `(content, scope)`.
pub type HighlightToken = (String, String);

/// A worker highlight response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightResponse {
    pub id: u64,
    pub key: String,
    pub language: String,
    pub reset: bool,
    pub stable: Vec<HighlightToken>,
    pub unstable: Vec<HighlightToken>,
}

/// The accumulated highlight state for a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerState {
    pub id: u64,
    pub generation: u32,
    pub language: String,
    pub stable: Vec<HighlightToken>,
    pub unstable: Vec<HighlightToken>,
}

/// Apply a worker response, accumulating stable tokens and replacing the tail.
pub fn apply_markdown_worker_response(
    state: Option<WorkerState>,
    response: HighlightResponse,
) -> WorkerState {
    match state {
        None => WorkerState {
            id: response.id,
            generation: 1,
            language: response.language,
            stable: response.stable,
            unstable: response.unstable,
        },
        Some(mut state) => {
            if response.id < state.id {
                return state;
            }
            if response.reset {
                state.generation += 1;
                state.stable = response.stable;
            } else {
                state.stable.extend(response.stable);
            }
            state.unstable = response.unstable;
            state.language = response.language;
            state.id = response.id;
            state
        }
    }
}

/// Whether the completed worker state for `id` should be released.
pub fn should_release_markdown_worker_state(completed: bool, id: u64, latest_id: u64) -> bool {
    completed && id == latest_id
}

/// Build a block key for the given owner and optional message namespace.
pub fn markdown_block_key(
    owner: &str,
    namespace: Option<&str>,
    index: usize,
    format: &str,
) -> String {
    match namespace {
        Some(namespace) => format!("{owner}:{namespace}:{index}:{format}"),
        None => format!("{owner}:block:{index}"),
    }
}
