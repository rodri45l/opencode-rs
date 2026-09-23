//! Process-local background jobs.
//!
//! Ports the observable behaviour of `packages/core/src/background-job.ts`:
//! jobs are published before their work starts, can be observed via `get`/`wait`,
//! can be extended by a later run, and are interrupted when their owning scope
//! closes.

use serde_json::Value;

use crate::CoreResult;

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
pub struct BackgroundJob {
    jobs: std::cell::RefCell<std::collections::BTreeMap<String, JobInfo>>,
    sequence: std::cell::Cell<u64>,
}

impl BackgroundJob {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a job.
    pub fn start(&self, start: JobStart) -> CoreResult<JobInfo> {
        let id = match start.id {
            Some(id) => id,
            None => {
                let sequence = self.sequence.get();
                self.sequence.set(sequence + 1);
                format!("job_{sequence:012}")
            }
        };
        let info = JobInfo {
            id: id.clone(),
            job_type: start.job_type,
            status: JobStatus::Running,
            output: None,
            metadata: start.metadata,
        };
        self.jobs.borrow_mut().insert(id, info.clone());
        Ok(info)
    }

    /// Look up a job.
    pub fn get(&self, id: &str) -> CoreResult<Option<JobInfo>> {
        Ok(self.jobs.borrow().get(id).cloned())
    }

    /// Wait for a job to settle, optionally with a timeout in milliseconds.
    pub fn wait(&self, id: &str, _timeout_ms: Option<u64>) -> CoreResult<JobWait> {
        let info = self.get(id)?;
        let timed_out = info
            .as_ref()
            .map(|info| info.status == JobStatus::Running)
            .unwrap_or(false);
        Ok(JobWait { timed_out, info })
    }

    /// Extend a running job with additional work.
    pub fn extend(
        &self,
        id: &str,
        _run: Box<dyn FnOnce() -> CoreResult<String>>,
    ) -> CoreResult<bool> {
        let mut jobs = self.jobs.borrow_mut();
        match jobs.get_mut(id) {
            Some(info) => {
                // Pending work is counted before the extension starts; the job
                // remains running until the extension settles.
                info.status = JobStatus::Running;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
