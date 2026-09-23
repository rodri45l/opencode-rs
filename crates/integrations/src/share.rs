//! In-memory enterprise share service.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/enterprise/test/core/share.test.ts` (upstream 18ef3cc). The
//! reference service is an Effect module over global storage; this port keeps
//! the create/remove/sync/data semantics on an owned instance so tests are
//! isolated. The legacy `share_event` migration case is deferred (see
//! docs/TEST-PORT.md).

use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Errors raised by the share service.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShareError {
    #[error("share not found")]
    NotFound,
    #[error("invalid share secret")]
    InvalidSecret,
}

/// A public share handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareInfo {
    pub id: String,
    pub session_id: String,
    pub secret: String,
}

/// The id/secret pair used to mutate an existing share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareRef {
    pub id: String,
    pub secret: String,
}

/// One piece of share data (session, message, or part).
#[derive(Debug, Clone, PartialEq)]
pub struct ShareData {
    pub r#type: String,
    pub data: Value,
}

impl ShareData {
    pub fn new(kind: &str, data: Value) -> Self {
        Self {
            r#type: kind.to_string(),
            data,
        }
    }
}

#[derive(Debug, Default)]
struct ShareRecord {
    session_id: String,
    secret: String,
    data: Vec<ShareData>,
}

/// An owned in-memory share service.
#[derive(Debug, Default)]
pub struct Share {
    records: BTreeMap<String, ShareRecord>,
}

fn unique_token() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos:x}{sequence:x}")
}

fn new_secret() -> String {
    format!("share_{}", unique_token())
}

impl Share {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a share for a session, returning its id and secret.
    pub fn create(&mut self, session_id: &str) -> ShareInfo {
        let id = format!("share_{}", unique_token());
        let secret = new_secret();
        self.records.insert(
            id.clone(),
            ShareRecord {
                session_id: session_id.to_string(),
                secret: secret.clone(),
                data: Vec::new(),
            },
        );
        ShareInfo {
            id,
            session_id: session_id.to_string(),
            secret,
        }
    }

    /// Look up a share by id.
    pub fn get(&self, id: &str) -> Option<ShareInfo> {
        self.records.get(id).map(|record| ShareInfo {
            id: id.to_string(),
            session_id: record.session_id.clone(),
            secret: record.secret.clone(),
        })
    }

    /// Remove a share as its owner, validating the secret.
    pub fn remove(&mut self, share: &ShareRef) -> Result<(), ShareError> {
        match self.records.get(&share.id) {
            None => Err(ShareError::NotFound),
            Some(record) if record.secret != share.secret => Err(ShareError::InvalidSecret),
            Some(_) => {
                self.records.remove(&share.id);
                Ok(())
            }
        }
    }

    /// Remove a share without a secret (admin path).
    pub fn remove_admin(&mut self, id: &str) {
        self.records.remove(id);
    }

    /// Append (or replace, by `data.id`) share data.
    pub fn sync(&mut self, share: &ShareRef, data: &[ShareData]) -> Result<(), ShareError> {
        let record = match self.records.get_mut(&share.id) {
            None => return Err(ShareError::NotFound),
            Some(record) if record.secret != share.secret => return Err(ShareError::InvalidSecret),
            Some(record) => record,
        };
        for item in data {
            let existing = item.data.get("id").and_then(Value::as_str).and_then(|id| {
                record.data.iter().position(|candidate| {
                    candidate.data.get("id").and_then(Value::as_str) == Some(id)
                })
            });
            match existing {
                Some(index) => record.data[index] = item.clone(),
                None => record.data.push(item.clone()),
            }
        }
        Ok(())
    }

    /// Read the accumulated data for a share.
    pub fn data(&self, id: &str) -> Vec<ShareData> {
        self.records
            .get(id)
            .map(|record| record.data.clone())
            .unwrap_or_default()
    }
}
