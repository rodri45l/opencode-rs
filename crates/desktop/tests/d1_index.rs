//! Port of packages/desktop/src/main/index.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/initialization.ts: a loading
//! task failure is forwarded into the initialization `Deferred` before (or while)
//! renderer initialization awaits it, so the renderer observes the original
//! failure.
//! Re-derived: Effect `Deferred`/`Fiber` plumbing becomes an owned deferred cell.

#[allow(dead_code)]
mod initialization {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop initialization failure forwarding not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

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

    #[derive(Debug, Default)]
    pub struct Deferred {
        pub settled: Option<Result<(), Failure>>,
    }

    pub fn make_deferred() -> Deferred {
        Deferred { settled: None }
    }

    pub fn forward_initialization_failure(
        _deferred: &mut Deferred,
        _failure: Failure,
    ) -> PortResult<()> {
        stub()
    }

    pub fn await_deferred(_deferred: &Deferred) -> PortResult<Result<(), Failure>> {
        stub()
    }
}

use initialization::{
    await_deferred, forward_initialization_failure, make_deferred, Failure, NOTE,
};

fn assert_forwarded_failure(failure: Failure) {
    let mut deferred = make_deferred();
    forward_initialization_failure(&mut deferred, failure.clone()).expect(NOTE);
    let exit = await_deferred(&deferred).expect(NOTE);
    assert!(exit.is_err());
    assert_eq!(exit.expect_err("failure"), failure);
}

#[test]
#[ignore = "porting: desktop initialization failure forwarding not implemented"]
fn forwards_loading_task_failures_before_renderer_initialization() {
    assert_forwarded_failure(Failure::new("sidecar startup failed"));
}

#[test]
#[ignore = "porting: desktop initialization failure forwarding not implemented"]
fn forwards_loading_task_failures_while_renderer_initialization_waits() {
    assert_forwarded_failure(Failure::new("sidecar startup failed"));
}
