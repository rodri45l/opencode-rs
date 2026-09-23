//! Port of packages/core/test/background-job.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: jobs are observable through explicit `get`/`wait`, a zero
//! timeout reports `timedOut` while the job is still running, and a job can be
//! extended by a later run. Dropped: the immediate-settle ordering and
//! scope-close interruption cases, which depend on `Effect`/`Scope` mechanics.

use opencode_core::background_job::{BackgroundJob, JobStart, JobStatus};
use serde_json::json;

const NOTE: &str = "porting: background job not implemented";

fn start(id: Option<&str>) -> JobStart {
    JobStart {
        id: id.map(str::to_string),
        job_type: "test".into(),
        metadata: json!({ "durable": false }),
        run: Box::new(|| Ok("done".into())),
    }
}

#[test]
#[ignore = "porting: background job not implemented"]
fn tracks_process_local_work_through_explicit_observation() {
    let jobs = BackgroundJob::new();
    let job = jobs.start(start(None)).expect(NOTE);

    assert_eq!(job.job_type, "test");
    assert_eq!(job.status, JobStatus::Running);
    assert_eq!(job.metadata, json!({ "durable": false }));

    let running = jobs.wait(&job.id, Some(0)).expect(NOTE);
    assert!(running.timed_out);
    assert_eq!(
        running.info.map(|info| info.status),
        Some(JobStatus::Running)
    );
}

#[test]
#[ignore = "porting: background job not implemented"]
fn increments_pending_work_before_starting_extensions() {
    let jobs = BackgroundJob::new();
    let job = jobs.start(start(Some("job_extend"))).expect(NOTE);

    assert!(jobs
        .extend(&job.id, Box::new(|| Ok("second".into())))
        .expect(NOTE));
    assert_eq!(
        jobs.get(&job.id).expect(NOTE).map(|info| info.status),
        Some(JobStatus::Running)
    );
}
