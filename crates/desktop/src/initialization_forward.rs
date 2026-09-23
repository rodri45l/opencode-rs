//! Initialization failure forwarding.
//!
//! Port of `packages/desktop/src/main/initialization.ts` (upstream 18ef3cc):
//! a loading-task failure is forwarded into the initialization deferred so the
//! renderer observes the original failure.

/// A forwarded initialization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub message: String,
}

impl Failure {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

/// A one-shot deferred cell.
#[derive(Debug, Default)]
pub struct Deferred {
    pub settled: Option<Result<(), Failure>>,
}

/// Create an unsettled deferred.
pub fn make_deferred() -> Deferred {
    Deferred { settled: None }
}

/// Forward a failure into the deferred.
pub fn forward_initialization_failure(deferred: &mut Deferred, failure: Failure) {
    deferred.settled = Some(Err(failure));
}

/// Await the deferred, defaulting to success when unsettled.
pub fn await_deferred(deferred: &Deferred) -> Result<(), Failure> {
    deferred.settled.clone().unwrap_or(Ok(()))
}
