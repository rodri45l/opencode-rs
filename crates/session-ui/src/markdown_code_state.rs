//! Highlight token reset policy.
//!
//! Port of packages/session-ui/src/components/markdown-code-state.ts behaviour
//! (upstream 18ef3cc).

/// A snapshot of the worker-highlighted code tokens for one block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeTokenState {
    pub language: String,
    pub generation: u32,
    pub stable_count: usize,
    pub unstable: Vec<(String, String)>,
    pub raw: String,
}

/// Whether the previous token state must be discarded for `next`.
///
/// Append-only streaming updates keep their tokens; a non-prefix replacement
/// (or a generation change) resets them.
pub fn should_reset_code_tokens(previous: &CodeTokenState, next: &CodeTokenState) -> bool {
    next.generation != previous.generation || !next.raw.starts_with(&previous.raw)
}
