//! Port of packages/core/test/session-run-coordinator.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: concurrent resumes for one key join a single drain, a wake
//! on an idle key starts a non-forced drain, wakes during an active drain coalesce
//! into one follow-up, resumes registered during interruption cleanup start a
//! forced drain, `active` snapshots only running keys in start order, failure and
//! defect clean the active set, and an interrupt clears any pending wake.
//! Re-derived: Effect fibers/`Deferred`/`Scope` interleavings are replaced by an
//! explicit deterministic scheduler (`interrupt_begin`/`interrupt_finish`). The
//! fiber-only cases — joined-waiter interruption and the 20k synchronous
//! self-wake trampoline — are not reproducible without a scheduler and are
//! skipped.

#![allow(dead_code)]

const NOTE: &str = "porting: session run coordinator not implemented";

mod local {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    /// How a drain finished.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DrainOutcome {
        Complete,
        Fail,
        Defect,
    }

    /// The result of starting or joining a drain.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum RunStart {
        Started(usize),
        Joined,
    }

    /// A recorded drain invocation.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RunRecord {
        pub key: String,
        pub force: bool,
    }

    #[derive(Debug, Default)]
    pub struct RunCoordinator {
        runs: Vec<RunRecord>,
    }

    impl RunCoordinator {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn run(&mut self, _key: &str, _force: bool) -> Result<RunStart, PortError> {
            Err(PortError::NotImplemented("session run coordinator"))
        }

        pub fn wake(&mut self, _key: &str) -> Result<Option<RunStart>, PortError> {
            Err(PortError::NotImplemented("session run coordinator"))
        }

        pub fn interrupt_begin(&mut self, _key: &str) -> Result<(), PortError> {
            Err(PortError::NotImplemented("session run coordinator"))
        }

        pub fn interrupt_finish(&mut self, _key: &str) -> Result<Vec<RunStart>, PortError> {
            Err(PortError::NotImplemented("session run coordinator"))
        }

        pub fn complete(
            &mut self,
            _key: &str,
            _outcome: DrainOutcome,
        ) -> Result<Vec<RunStart>, PortError> {
            Err(PortError::NotImplemented("session run coordinator"))
        }

        pub fn active(&self) -> Vec<String> {
            Vec::new()
        }

        pub fn runs(&self) -> Vec<RunRecord> {
            self.runs.clone()
        }

        pub fn forces(&self) -> Vec<bool> {
            self.runs.iter().map(|run| run.force).collect()
        }
    }
}

use local::{DrainOutcome, RunCoordinator, RunStart};

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn joins_concurrent_resumes_for_one_key() {
    let mut coordinator = RunCoordinator::new();
    assert_eq!(
        coordinator.run("session", true).expect(NOTE),
        RunStart::Started(0)
    );
    assert_eq!(
        coordinator.run("session", false).expect(NOTE),
        RunStart::Joined
    );
    assert_eq!(coordinator.runs().len(), 1);
    assert_eq!(coordinator.active(), vec!["session".to_string()]);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn joins_a_wake_started_execution_without_forcing_a_successor() {
    let mut coordinator = RunCoordinator::new();
    coordinator.wake("session").expect(NOTE);
    coordinator.run("session", true).expect(NOTE);
    coordinator
        .complete("session", DrainOutcome::Complete)
        .expect(NOTE);
    assert_eq!(coordinator.forces(), vec![false]);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn starts_execution_when_woken_while_idle() {
    let mut coordinator = RunCoordinator::new();
    coordinator.wake("session").expect(NOTE);
    assert_eq!(coordinator.active(), vec!["session".to_string()]);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn snapshots_only_active_executions() {
    let mut coordinator = RunCoordinator::new();
    assert!(coordinator.active().is_empty());
    coordinator.run("first", true).expect(NOTE);
    assert_eq!(coordinator.active(), vec!["first".to_string()]);
    coordinator.run("second", true).expect(NOTE);
    assert_eq!(
        coordinator.active(),
        vec!["first".to_string(), "second".to_string()]
    );
    coordinator
        .complete("first", DrainOutcome::Complete)
        .expect(NOTE);
    assert_eq!(coordinator.active(), vec!["second".to_string()]);
    coordinator
        .complete("second", DrainOutcome::Complete)
        .expect(NOTE);
    assert!(coordinator.active().is_empty());
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn cleans_active_executions_after_failure_and_defect() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("failure", true).expect(NOTE);
    coordinator
        .complete("failure", DrainOutcome::Fail)
        .expect(NOTE);
    assert!(coordinator.active().is_empty());
    coordinator.run("defect", true).expect(NOTE);
    coordinator
        .complete("defect", DrainOutcome::Defect)
        .expect(NOTE);
    assert!(coordinator.active().is_empty());
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn starts_a_resume_registered_during_interruption_cleanup() {
    let mut coordinator = RunCoordinator::new();
    coordinator.wake("session").expect(NOTE);
    coordinator.interrupt_begin("session").expect(NOTE);
    coordinator.run("session", true).expect(NOTE);
    coordinator.interrupt_finish("session").expect(NOTE);
    assert_eq!(coordinator.forces(), vec![false, true]);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn runs_a_wake_registered_during_interruption_cleanup() {
    let mut coordinator = RunCoordinator::new();
    coordinator.wake("session").expect(NOTE);
    coordinator.interrupt_begin("session").expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator.interrupt_finish("session").expect(NOTE);
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn coalesces_wakes_received_during_active_execution() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("session", true).expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator
        .complete("session", DrainOutcome::Complete)
        .expect(NOTE);
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn runs_again_when_woken_during_the_follow_up() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("session", true).expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator
        .complete("session", DrainOutcome::Complete)
        .expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator
        .complete("session", DrainOutcome::Complete)
        .expect(NOTE);
    assert_eq!(coordinator.runs().len(), 3);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn does_nothing_when_interrupted_while_idle() {
    let mut coordinator = RunCoordinator::new();
    coordinator.interrupt_begin("session").expect(NOTE);
    coordinator.interrupt_finish("session").expect(NOTE);
    assert!(coordinator.active().is_empty());
    assert!(coordinator.runs().is_empty());
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn interrupts_active_execution_and_clears_its_pending_wake() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("session", true).expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator.interrupt_begin("session").expect(NOTE);
    coordinator.interrupt_finish("session").expect(NOTE);
    assert!(coordinator.active().is_empty());
    assert_eq!(coordinator.runs().len(), 1);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn starts_one_follow_up_when_a_wake_races_with_failure() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("session", true).expect(NOTE);
    coordinator.wake("session").expect(NOTE);
    coordinator
        .complete("session", DrainOutcome::Fail)
        .expect(NOTE);
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
#[ignore = "porting: session run coordinator not implemented"]
fn runs_different_keys_concurrently() {
    let mut coordinator = RunCoordinator::new();
    coordinator.run("first", true).expect(NOTE);
    coordinator.run("second", true).expect(NOTE);
    assert_eq!(
        coordinator.active(),
        vec!["first".to_string(), "second".to_string()]
    );
    assert_eq!(coordinator.runs().len(), 2);
}
