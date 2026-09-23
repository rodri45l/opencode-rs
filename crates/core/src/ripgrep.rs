//! Ripgrep-backed filesystem search.
//!
//! Ports the observable behaviour of `packages/core/src/ripgrep.ts`: `glob`
//! returns matching paths relative to the working directory and `grep` filters
//! by an include glob and reports line submatches.

use crate::{CoreError, CoreResult};

/// A path match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobMatch {
    /// Path relative to the search root.
    pub path: String,
}

/// A matched line fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submatch {
    /// Matched text.
    pub text: String,
}

/// A grep match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepMatch {
    /// The matching file.
    pub entry: GlobMatch,
    /// Line submatches.
    pub submatches: Vec<Submatch>,
}

/// Ripgrep search entry point.
#[derive(Debug, Default)]
pub struct Ripgrep;

impl Ripgrep {
    /// Glob files under `cwd` matching `pattern`.
    pub fn glob(_cwd: &str, _pattern: &str, _limit: usize) -> CoreResult<Vec<GlobMatch>> {
        Err(CoreError::NotImplemented("ripgrep::Ripgrep::glob"))
    }

    /// Grep files under `cwd` for `pattern`, filtered by `include`.
    pub fn grep(
        _cwd: &str,
        _pattern: &str,
        _include: &str,
        _limit: usize,
    ) -> CoreResult<Vec<GrepMatch>> {
        Err(CoreError::NotImplemented("ripgrep::Ripgrep::grep"))
    }

    /// The line preview length before an ellipsis is appended.
    pub const MAX_PREVIEW_CHARS: usize = 1999;

    /// Whether a relative path is always excluded from search results (the `.git`
    /// metadata directory).
    pub fn is_ignored(_path: &str) -> CoreResult<bool> {
        Err(CoreError::NotImplemented("ripgrep::Ripgrep::is_ignored"))
    }

    /// Truncate a matched line to [`Self::MAX_PREVIEW_CHARS`] characters and
    /// append an ellipsis, never splitting a surrogate pair.
    pub fn preview_line(_line: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented("ripgrep::Ripgrep::preview_line"))
    }
}
