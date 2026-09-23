//! In-memory key/value storage backing the enterprise share surface.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/enterprise/test/core/storage.test.ts` (upstream 18ef3cc). The
//! reference service is an async global store; this port keeps the same
//! list/write/read/remove semantics on an owned instance so tests are isolated.

use serde_json::Value;
use std::collections::BTreeMap;

/// Options for [`Storage::list`].
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub prefix: Vec<String>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<usize>,
}

/// A path-addressable JSON store.
#[derive(Debug, Default)]
pub struct Storage {
    entries: BTreeMap<Vec<String>, Value>,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Write a JSON value at `path`.
    pub fn write(&mut self, path: &[&str], value: Value) {
        self.entries.insert(
            path.iter().map(|segment| (*segment).to_string()).collect(),
            value,
        );
    }

    /// Read the JSON value at `path`, if present.
    pub fn read(&self, path: &[&str]) -> Option<&Value> {
        let key: Vec<String> = path.iter().map(|segment| (*segment).to_string()).collect();
        self.entries.get(&key)
    }

    /// Remove the value at `path`.
    pub fn remove(&mut self, path: &[&str]) {
        let key: Vec<String> = path.iter().map(|segment| (*segment).to_string()).collect();
        self.entries.remove(&key);
    }

    /// List stored paths under `prefix`, honouring `after`/`before`/`limit`.
    pub fn list(&self, options: &ListOptions) -> Vec<Vec<String>> {
        let mut paths: Vec<Vec<String>> = self
            .entries
            .keys()
            .filter(|key| key.starts_with(&options.prefix))
            .filter(|key| {
                let leaf = key.last().map(String::as_str).unwrap_or("");
                options.after.as_deref().is_none_or(|after| leaf > after)
                    && options.before.as_deref().is_none_or(|before| leaf < before)
            })
            .cloned()
            .collect();
        paths.sort();
        if let Some(limit) = options.limit {
            paths.truncate(limit);
        }
        paths
    }
}
