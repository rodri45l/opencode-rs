//! Session compaction overflow detection.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/opencode/test/session/compaction.test.ts` (upstream 18ef3cc):
//! the `isOverflow` cases only. The reference also exercises
//! `SessionCompaction.create`/`prune`/`process`, which require the live
//! session database, projector, and LLM stream; those are dropped here and
//! covered once those services are ported.

/// Token accounting for the most recent assistant response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenCounts {
    /// Prompt tokens.
    pub input: u64,
    /// Completion tokens.
    pub output: u64,
    /// Reasoning tokens.
    pub reasoning: u64,
    /// Cache-read tokens, counted toward the window.
    pub cache_read: u64,
    /// Cache-write tokens (not counted toward the window).
    pub cache_write: u64,
}

/// Model limits relevant to overflow detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactionModel {
    /// Total context window.
    pub context: u64,
    /// Explicit input cap, when the provider publishes one.
    pub input_limit: Option<u64>,
    /// Maximum output tokens.
    pub output_limit: u64,
}

/// Whether the conversation must be compacted before the next request.
///
/// `auto_enabled` mirrors `compaction.auto` in config; when disabled the
/// reference always returns `false`.
pub fn is_overflow(tokens: TokenCounts, model: CompactionModel, auto_enabled: bool) -> bool {
    if !auto_enabled || model.context == 0 {
        return false;
    }
    let count = tokens
        .input
        .saturating_add(tokens.output)
        .saturating_add(tokens.reasoning)
        .saturating_add(tokens.cache_read);
    let headroom = model.context.saturating_sub(model.output_limit);
    let usable = match model.input_limit {
        Some(limit) if limit > 0 => limit.min(headroom),
        _ => headroom,
    };
    count > usable
}
