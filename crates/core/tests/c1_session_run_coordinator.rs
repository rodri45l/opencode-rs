//! Port of packages/core/test/session-run-coordinator.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: concurrent resumes for one key join a single drain, a wake
//! on an idle key starts a non-forced drain, wakes during an active drain coalesce
//! into one follow-up, resumes registered during interruption cleanup start a
//! forced drain, `active` snapshots only running keys in start order, failure and
//! defect clean the active set, and an interrupt clears any pending wake.
//! Re-derived: Effect fibers/`Deferred`/`Scope` interleavings are replaced by an
//! explicit deterministic scheduler in `opencode_core::run_coordinator`. The
//! fiber-only cases — joined-waiter interruption and the 20k synchronous
//! self-wake trampoline — are not reproducible without a scheduler and are
//! skipped.

use opencode_core::run_coordinator::{DrainOutcome, RunCoordinator, RunStart};

#[test]
fn joins_concurrent_resumes_for_one_key() {
    let mut coordinator = RunCoordinator::new();
    assert_eq!(coordinator.run("session", true), RunStart::Started(0));
    assert_eq!(coordinator.run("session", false), RunStart::Joined);
    assert_eq!(coordinator.runs().len(), 1);
    assert_eq!(coordinator.active(), vec!["session".to_string()]);
}

#[test]
fn joins_a_wake_started_execution_without_forcing_a_successor() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.wake("session");
    let _ = coordinator.run("session", true);
    let _ = coordinator.complete("session", DrainOutcome::Complete);
    assert_eq!(coordinator.forces(), vec![false]);
}

#[test]
fn starts_execution_when_woken_while_idle() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.wake("session");
    assert_eq!(coordinator.active(), vec!["session".to_string()]);
}

#[test]
fn snapshots_only_active_executions() {
    let mut coordinator = RunCoordinator::new();
    assert!(coordinator.active().is_empty());
    let _ = coordinator.run("first", true);
    assert_eq!(coordinator.active(), vec!["first".to_string()]);
    let _ = coordinator.run("second", true);
    assert_eq!(
        coordinator.active(),
        vec!["first".to_string(), "second".to_string()]
    );
    let _ = coordinator.complete("first", DrainOutcome::Complete);
    assert_eq!(coordinator.active(), vec!["second".to_string()]);
    let _ = coordinator.complete("second", DrainOutcome::Complete);
    assert!(coordinator.active().is_empty());
}

#[test]
fn cleans_active_executions_after_failure_and_defect() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("failure", true);
    let _ = coordinator.complete("failure", DrainOutcome::Fail);
    assert!(coordinator.active().is_empty());
    let _ = coordinator.run("defect", true);
    let _ = coordinator.complete("defect", DrainOutcome::Defect);
    assert!(coordinator.active().is_empty());
}

#[test]
fn starts_a_resume_registered_during_interruption_cleanup() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.wake("session");
    coordinator.interrupt_begin("session");
    let _ = coordinator.run("session", true);
    let _ = coordinator.interrupt_finish("session");
    assert_eq!(coordinator.forces(), vec![false, true]);
}

#[test]
fn runs_a_wake_registered_during_interruption_cleanup() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.wake("session");
    coordinator.interrupt_begin("session");
    let _ = coordinator.wake("session");
    let _ = coordinator.interrupt_finish("session");
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
fn coalesces_wakes_received_during_active_execution() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("session", true);
    let _ = coordinator.wake("session");
    let _ = coordinator.wake("session");
    let _ = coordinator.wake("session");
    let _ = coordinator.complete("session", DrainOutcome::Complete);
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
fn runs_again_when_woken_during_the_follow_up() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("session", true);
    let _ = coordinator.wake("session");
    let _ = coordinator.complete("session", DrainOutcome::Complete);
    let _ = coordinator.wake("session");
    let _ = coordinator.complete("session", DrainOutcome::Complete);
    assert_eq!(coordinator.runs().len(), 3);
}

#[test]
fn does_nothing_when_interrupted_while_idle() {
    let mut coordinator = RunCoordinator::new();
    coordinator.interrupt_begin("session");
    let _ = coordinator.interrupt_finish("session");
    assert!(coordinator.active().is_empty());
    assert!(coordinator.runs().is_empty());
}

#[test]
fn interrupts_active_execution_and_clears_its_pending_wake() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("session", true);
    let _ = coordinator.wake("session");
    coordinator.interrupt_begin("session");
    let _ = coordinator.interrupt_finish("session");
    assert!(coordinator.active().is_empty());
    assert_eq!(coordinator.runs().len(), 1);
}

#[test]
fn starts_one_follow_up_when_a_wake_races_with_failure() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("session", true);
    let _ = coordinator.wake("session");
    let _ = coordinator.complete("session", DrainOutcome::Fail);
    assert_eq!(coordinator.runs().len(), 2);
}

#[test]
fn runs_different_keys_concurrently() {
    let mut coordinator = RunCoordinator::new();
    let _ = coordinator.run("first", true);
    let _ = coordinator.run("second", true);
    assert_eq!(
        coordinator.active(),
        vec!["first".to_string(), "second".to_string()]
    );
    assert_eq!(coordinator.runs().len(), 2);
}
