//! Markdown worker transport queue.
//!
//! Derived from `packages/session-ui/src/components/markdown-worker-transport.ts`
//! (upstream 18ef3cc): one request is posted per key and only the latest queued
//! snapshot is retained; responses for a disposed request are ignored; disposing
//! drops queued snapshots.

use std::collections::BTreeMap;
use std::fmt;

/// Error raised by the worker transport.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the worker transport.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui markdown worker transport";

/// One in-flight or queued request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerRequest {
    pub id: i64,
    pub key: String,
}

/// A queue that posts at most one request per key and supersedes stale ones.
pub struct WorkerTransport {
    post: Box<dyn FnMut(&WorkerRequest)>,
    supersede: Box<dyn FnMut(&WorkerRequest)>,
    active: BTreeMap<String, WorkerRequest>,
    queued: BTreeMap<String, WorkerRequest>,
}

impl WorkerTransport {
    /// Build a transport from post and supersede callbacks.
    pub fn new(
        post: impl FnMut(&WorkerRequest) + 'static,
        supersede: impl FnMut(&WorkerRequest) + 'static,
    ) -> Self {
        Self {
            post: Box::new(post),
            supersede: Box::new(supersede),
            active: BTreeMap::new(),
            queued: BTreeMap::new(),
        }
    }

    /// Post or queue a request.
    pub fn send(&mut self, request: WorkerRequest) -> PortResult<()> {
        if !self.active.contains_key(&request.key) {
            self.active.insert(request.key.clone(), request.clone());
            (self.post)(&request);
            return Ok(());
        }
        if let Some(previous) = self.queued.get(&request.key) {
            (self.supersede)(previous);
        }
        self.queued.insert(request.key.clone(), request);
        Ok(())
    }

    /// Complete the active request for a key, posting the queued one.
    pub fn complete(&mut self, key: &str, id: i64) -> PortResult<()> {
        if self.active.get(key).map(|request| request.id) != Some(id) {
            return Ok(());
        }
        self.active.remove(key);
        if let Some(next) = self.queued.remove(key) {
            self.active.insert(key.to_string(), next.clone());
            (self.post)(&next);
        }
        Ok(())
    }

    /// Dispose a key, dropping its active and queued requests.
    pub fn dispose(&mut self, key: &str) -> PortResult<()> {
        self.active.remove(key);
        if let Some(request) = self.queued.remove(key) {
            (self.supersede)(&request);
        }
        Ok(())
    }

    /// The number of queued snapshots.
    pub fn queued(&self) -> PortResult<usize> {
        Ok(self.queued.len())
    }
}
