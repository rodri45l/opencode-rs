//! Markdown code-token reset rule.
//!
//! Derived from `packages/session-ui/src/components/markdown-code-state.ts`
//! (upstream 18ef3cc).

use std::fmt;

/// Error raised by the code-state helper.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the code-state helper.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui markdown code state";

/// Rendered code-token state for one code block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeTokenState {
    pub language: String,
    pub generation: i64,
    pub stable_count: i64,
    pub unstable: Vec<(String, String)>,
    pub raw: String,
}

/// Whether the next state must discard the previous token state.
pub fn should_reset_code_tokens(
    previous: &CodeTokenState,
    next: &CodeTokenState,
) -> PortResult<bool> {
    let compatible = previous.language == next.language
        && previous.generation == next.generation
        && next.stable_count >= previous.stable_count
        && next.raw.starts_with(&previous.raw);
    Ok(!compatible)
}
