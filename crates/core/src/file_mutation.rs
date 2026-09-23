//! Guarded file mutations.
//!
//! Ports the observable behaviour of `packages/core/src/file-mutation.ts` and
//! `location-mutation.ts`: resolved targets carry a canonical path and a
//! resource name, writes report whether the target existed, conditional
//! creates/writes fail on appearing or stale targets, and text writes preserve
//! exactly one BOM.

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// A resolved mutation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTarget {
    /// Canonical absolute path.
    pub canonical: AbsolutePath,
    /// Resource name relative to the location (or the path for externals).
    pub resource: String,
}

/// Resolves paths into mutation targets.
#[derive(Debug, Default)]
pub struct LocationMutation;

impl LocationMutation {
    /// Resolve `path` against the active location.
    pub fn resolve(&self, _path: &str) -> CoreResult<ResolvedTarget> {
        Err(CoreError::NotImplemented(
            "file_mutation::LocationMutation::resolve",
        ))
    }
}

/// The outcome of a file mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMutationResult {
    /// `write` or `remove`.
    pub operation: String,
    /// Canonical target path.
    pub target: AbsolutePath,
    /// Resource name.
    pub resource: String,
    /// Whether the target existed before the mutation.
    pub existed: bool,
}

/// Guarded file mutations.
#[derive(Debug, Default)]
pub struct FileMutation;

impl FileMutation {
    /// Write `content` to `target`.
    pub fn write(
        &self,
        _target: &ResolvedTarget,
        _content: &str,
    ) -> CoreResult<FileMutationResult> {
        Err(CoreError::NotImplemented(
            "file_mutation::FileMutation::write",
        ))
    }

    /// Create `target`, failing if it appeared after resolution.
    pub fn create(
        &self,
        _target: &ResolvedTarget,
        _content: &str,
    ) -> CoreResult<FileMutationResult> {
        Err(CoreError::NotImplemented(
            "file_mutation::FileMutation::create",
        ))
    }

    /// Remove `target`, whether or not it exists.
    pub fn remove(&self, _target: &ResolvedTarget) -> CoreResult<FileMutationResult> {
        Err(CoreError::NotImplemented(
            "file_mutation::FileMutation::remove",
        ))
    }

    /// Write text, preserving exactly one BOM.
    pub fn write_text_preserving_bom(
        &self,
        _target: &ResolvedTarget,
        _content: &str,
    ) -> CoreResult<FileMutationResult> {
        Err(CoreError::NotImplemented(
            "file_mutation::FileMutation::write_text_preserving_bom",
        ))
    }

    /// Write only when the current bytes equal `expected`.
    pub fn write_if_unchanged(
        &self,
        _target: &ResolvedTarget,
        _expected: &[u8],
        _content: &str,
    ) -> CoreResult<FileMutationResult> {
        Err(CoreError::NotImplemented(
            "file_mutation::FileMutation::write_if_unchanged",
        ))
    }
}
