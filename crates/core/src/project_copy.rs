//! Project copy helpers.
//!
//! Ports the pure, decidable parts of `packages/core/src/project-copy.ts`:
//! strategy-id validation, duplicate/unavailable strategy errors, numbered copy
//! directory selection, and refresh diffs. The `ProjectCopy`/`Database`/`EventV2`
//! service wiring and live git worktrees are not reproduced here.

/// Error raised by project-copy helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyError {
    /// A strategy id is empty or has surrounding whitespace.
    InvalidStrategyId,
    /// The strategy id is already registered.
    DuplicateStrategy,
    /// The strategy id is not registered.
    StrategyUnavailable(String),
    /// No free copy directory could be found.
    DestinationExists(String),
}

impl std::fmt::Display for CopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyError::InvalidStrategyId => write!(f, "invalid strategy id"),
            CopyError::DuplicateStrategy => write!(f, "duplicate strategy"),
            CopyError::StrategyUnavailable(id) => write!(f, "strategy unavailable: {id}"),
            CopyError::DestinationExists(path) => write!(f, "destination exists: {path}"),
        }
    }
}

impl std::error::Error for CopyError {}

/// A validated strategy id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyId(pub String);

/// Parse and validate a strategy id (non-empty, no surrounding whitespace).
pub fn parse_strategy_id(input: &str) -> Result<StrategyId, CopyError> {
    if input.is_empty() || input.trim() != input {
        return Err(CopyError::InvalidStrategyId);
    }
    Ok(StrategyId(input.to_string()))
}

/// Register `id`, rejecting a duplicate.
pub fn register_strategy(known: &[String], id: &str) -> Result<(), CopyError> {
    if known.iter().any(|entry| entry == id) {
        return Err(CopyError::DuplicateStrategy);
    }
    Ok(())
}

/// Require that `id` is registered.
pub fn require_strategy(known: &[String], id: &str) -> Result<(), CopyError> {
    if known.iter().any(|entry| entry == id) {
        Ok(())
    } else {
        Err(CopyError::StrategyUnavailable(id.to_string()))
    }
}

/// The directory name for copy `index` (1-based).
pub fn candidate_directory_name(name: &str, index: u32) -> Result<String, CopyError> {
    if index <= 1 {
        Ok(name.to_string())
    } else {
        Ok(format!("{name}-{index}"))
    }
}

/// Resolve the first free copy directory, failing after ten conflicts.
pub fn resolve_directory(
    parent: &str,
    name: &str,
    exists: impl Fn(&str) -> bool,
) -> Result<String, CopyError> {
    let parent = parent.trim_end_matches('/');
    for index in 1..=10 {
        let candidate = candidate_directory_name(name, index)?;
        let path = format!("{parent}/{candidate}");
        if !exists(&path) {
            return Ok(path);
        }
    }
    let last = candidate_directory_name(name, 10)?;
    Err(CopyError::DestinationExists(format!("{parent}/{last}")))
}

/// The added/removed directories between the stored and present sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshDiff {
    /// Directories present but not stored.
    pub updated: Vec<String>,
    /// Directories stored but no longer present.
    pub removed: Vec<String>,
}

/// Compute a refresh diff.
pub fn refresh_diff(stored: &[&str], present: &[&str]) -> Result<RefreshDiff, CopyError> {
    let updated = present
        .iter()
        .filter(|path| !stored.contains(path))
        .map(|path| (*path).to_string())
        .collect();
    let removed = stored
        .iter()
        .filter(|path| !present.contains(path))
        .map(|path| (*path).to_string())
        .collect();
    Ok(RefreshDiff { updated, removed })
}
