//! Process-local background jobs.
//!
//! Ports the observable behaviour of `packages/core/src/background-job.ts`:
//! jobs are published before their work starts, can be observed via `get`/`wait`,
//! can be extended by a later run, and are interrupted when their owning scope
//! closes.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// Lifecycle status of a background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Work is in flight.
    Running,
    /// Work settled successfully.
    Completed,
    /// Work failed.
    Failed,
}

/// A background job's observable info.
#[derive(Debug, Clone, PartialEq)]
pub struct JobInfo {
    /// Job id.
    pub id: String,
    /// Job type.
    pub job_type: String,
    /// Lifecycle status.
    pub status: JobStatus,
    /// Settled output, when completed.
    pub output: Option<String>,
    /// Arbitrary metadata.
    pub metadata: Value,
}

/// A request to start a background job.
pub struct JobStart {
    /// Optional explicit id.
    pub id: Option<String>,
    /// Job type.
    pub job_type: String,
    /// Arbitrary metadata.
    pub metadata: Value,
    /// Work to run.
    pub run: Box<dyn FnOnce() -> CoreResult<String>>,
}

/// The result of waiting on a job.
#[derive(Debug, Clone, PartialEq)]
pub struct JobWait {
    /// Whether the wait timed out.
    pub timed_out: bool,
    /// The latest observable info.
    pub info: Option<JobInfo>,
}

/// Background job registry.
#[derive(Debug, Default)]
pub struct BackgroundJob;

impl BackgroundJob {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self
    }

    /// Start a job.
    pub fn start(&self, _start: JobStart) -> CoreResult<JobInfo> {
        Err(CoreError::NotImplemented(
            "background_job::BackgroundJob::start",
        ))
    }

    /// Look up a job.
    pub fn get(&self, _id: &str) -> CoreResult<Option<JobInfo>> {
        Err(CoreError::NotImplemented(
            "background_job::BackgroundJob::get",
        ))
    }

    /// Wait for a job to settle, optionally with a timeout in milliseconds.
    pub fn wait(&self, _id: &str, _timeout_ms: Option<u64>) -> CoreResult<JobWait> {
        Err(CoreError::NotImplemented(
            "background_job::BackgroundJob::wait",
        ))
    }

    /// Extend a running job with additional work.
    pub fn extend(
        &self,
        _id: &str,
        _run: Box<dyn FnOnce() -> CoreResult<String>>,
    ) -> CoreResult<bool> {
        Err(CoreError::NotImplemented(
            "background_job::BackgroundJob::extend",
        ))
    }
}
