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
    pub fn decode_list_input(value: &Value) -> CoreResult<ListInput> {
        let project_id = value
            .get("projectID")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::Invalid("list input missing projectID".into()))?;
        Ok(ListInput {
            project_id: project_id.to_string(),
        })
    }

    /// Decode the `list` output schema.
    pub fn decode_list_output(value: &Value) -> CoreResult<Vec<DirectoryEntry>> {
        let items = value
            .as_array()
            .ok_or_else(|| CoreError::Invalid("list output must be an array".into()))?;
        let mut entries = Vec::new();
        for item in items {
            let directory = item
                .get("directory")
                .and_then(Value::as_str)
                .ok_or_else(|| CoreError::Invalid("directory entry missing directory".into()))?;
            entries.push(DirectoryEntry {
                directory: directory.to_string(),
                strategy: item
                    .get("strategy")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            });
        }
        Ok(entries)
    }

    /// Insert a directory, returning whether the store changed.
    pub fn create(&mut self, input: CreateInput) -> CoreResult<bool> {
        let entries = self.entries.entry(input.project_id.clone()).or_default();
        match entries
            .iter_mut()
            .find(|entry| entry.directory == input.directory)
        {
            Some(existing) => match input.behavior {
                CreateBehavior::Ignore => Ok(false),
                CreateBehavior::Replace => {
                    if existing.strategy == input.strategy {
                        Ok(false)
                    } else {
                        existing.strategy = input.strategy;
                        Ok(true)
                    }
                }
            },
            None => {
                entries.push(DirectoryEntry {
                    directory: input.directory,
                    strategy: input.strategy,
                });
                Ok(true)
            }
        }
    }

    /// List a project's directories.
    pub fn list(&self, project_id: &str) -> CoreResult<Vec<DirectoryEntry>> {
        Ok(self.entries.get(project_id).cloned().unwrap_or_default())
    }
}
