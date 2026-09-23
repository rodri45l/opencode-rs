//! Session run coordinator (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/run-coordinator.ts`: concurrent resumes for one
//! key join a single drain, a wake on an idle key starts a non-forced drain,
//! wakes during an active drain coalesce into one follow-up, resumes registered
//! during interruption cleanup start a forced drain, `active` snapshots only
//! running keys in start order, failure and defect clean the active set, and an
//! interrupt clears any pending wake. The Effect fibers/`Deferred`/`Scope`
//! interleavings are replaced by an explicit deterministic scheduler.

use std::collections::BTreeMap;

/// How a drain finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainOutcome {
    /// The drain completed.
    Complete,
    /// The drain failed.
    Fail,
    /// The drain hit a defect.
    Defect,
}

/// The result of starting or joining a drain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStart {
    /// A new drain was started at the given index.
    Started(usize),
    /// An existing drain was joined.
    Joined,
}

/// A recorded drain invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    /// Drain key.
    pub key: String,
    /// Whether the drain was forced.
    pub force: bool,
}

#[derive(Debug, Default)]
struct KeyState {
    running: bool,
    pending_wake: bool,
    interrupting: bool,
    resume_force: Option<bool>,
}

/// The deterministic run coordinator.
#[derive(Debug, Default)]
pub struct RunCoordinator {
    runs: Vec<RunRecord>,
    active: Vec<String>,
    states: BTreeMap<String, KeyState>,
}

impl RunCoordinator {
    /// Create an empty coordinator.
    pub fn new() -> Self {
        Self::default()
    }

    fn start(&mut self, key: &str, force: bool) -> RunStart {
        let index = self.runs.len();
        self.runs.push(RunRecord {
            key: key.to_string(),
            force,
        });
        let state = self.states.entry(key.to_string()).or_default();
        state.running = true;
        if !self.active.iter().any(|active| active == key) {
            self.active.push(key.to_string());
        }
        RunStart::Started(index)
    }

    /// Start a drain for `key`, or join a running one.
    pub fn run(&mut self, key: &str, force: bool) -> RunStart {
        let state = self.states.entry(key.to_string()).or_default();
        if state.running {
            if state.interrupting {
                state.resume_force = Some(state.resume_force.unwrap_or(false) || force);
            }
            return RunStart::Joined;
        }
        self.start(key, force)
    }

    /// Wake `key`, starting a non-forced drain when idle.
    pub fn wake(&mut self, key: &str) -> Option<RunStart> {
        let state = self.states.entry(key.to_string()).or_default();
        if state.running {
            if state.interrupting {
                state.resume_force = Some(state.resume_force.unwrap_or(false));
                return None;
            }
            state.pending_wake = true;
            return None;
        }
        Some(self.start(key, false))
    }

    /// Mark `key` as interrupting.
    pub fn interrupt_begin(&mut self, key: &str) {
        self.states.entry(key.to_string()).or_default().interrupting = true;
    }

    /// Finish an interruption, starting a resume registered during cleanup.
    pub fn interrupt_finish(&mut self, key: &str) -> Vec<RunStart> {
        let resume = {
            let state = self.states.entry(key.to_string()).or_default();
            state.interrupting = false;
            state.pending_wake = false;
            state.resume_force.take()
        };
        match resume {
            Some(force) => vec![self.start(key, force)],
            None => {
                if let Some(state) = self.states.get_mut(key) {
                    state.running = false;
                }
                self.active.retain(|active| active != key);
                Vec::new()
            }
        }
    }

    /// Complete a drain, running a coalesced follow-up wake if present.
    pub fn complete(&mut self, key: &str, _outcome: DrainOutcome) -> Vec<RunStart> {
        let pending_wake = {
            let state = self.states.entry(key.to_string()).or_default();
            state.running = false;
            let pending = state.pending_wake;
            state.pending_wake = false;
            pending
        };
        if pending_wake {
            return vec![self.start(key, false)];
        }
        self.active.retain(|active| active != key);
        Vec::new()
    }

    /// Running keys in start order.
    pub fn active(&self) -> Vec<String> {
        self.active.clone()
    }

    /// All recorded drain invocations.
    pub fn runs(&self) -> Vec<RunRecord> {
        self.runs.clone()
    }

    /// The `force` flag of each recorded drain invocation.
    pub fn forces(&self) -> Vec<bool> {
        self.runs.iter().map(|run| run.force).collect()
    }
}
