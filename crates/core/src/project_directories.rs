//! Project directories.
//!
//! Ports the observable behaviour of
//! `packages/core/src/project/directories.ts`: directory list input/output
//! schemas decode plain data, `create` inserts once and ignores a conflicting
//! insert, and an explicit `replace` behavior updates the stored strategy.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A project directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryEntry {
    /// The directory path.
    pub directory: String,
    /// Optional creation strategy label.
    pub strategy: Option<String>,
}

/// Decoded `list` input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListInput {
    /// Project id.
    pub project_id: String,
}

/// The behavior when a directory already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateBehavior {
    /// Ignore the conflicting insert (default).
    #[default]
    Ignore,
    /// Replace the stored strategy.
    Replace,
}

/// A `create` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateInput {
    /// Project id.
    pub project_id: String,
    /// The directory path.
    pub directory: String,
    /// Optional strategy label.
    pub strategy: Option<String>,
    /// Conflict behavior.
    pub behavior: CreateBehavior,
}

/// Project directory storage.
#[derive(Debug, Default)]
pub struct ProjectDirectories {
    entries: BTreeMap<String, Vec<DirectoryEntry>>,
}

impl ProjectDirectories {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode the `list` input schema.
    pub fn decode_list_input(_value: &Value) -> CoreResult<ListInput> {
        Err(CoreError::NotImplemented(
            "project_directories::ProjectDirectories::decode_list_input",
        ))
    }

    /// Decode the `list` output schema.
    pub fn decode_list_output(_value: &Value) -> CoreResult<Vec<DirectoryEntry>> {
        Err(CoreError::NotImplemented(
            "project_directories::ProjectDirectories::decode_list_output",
        ))
    }

    /// Insert a directory, returning whether the store changed.
    pub fn create(&mut self, _input: CreateInput) -> CoreResult<bool> {
        let _ = &self.entries;
        Err(CoreError::NotImplemented(
            "project_directories::ProjectDirectories::create",
        ))
    }

    /// List a project's directories.
    pub fn list(&self, _project_id: &str) -> CoreResult<Vec<DirectoryEntry>> {
        let _ = &self.entries;
        Err(CoreError::NotImplemented(
            "project_directories::ProjectDirectories::list",
        ))
    }
}
