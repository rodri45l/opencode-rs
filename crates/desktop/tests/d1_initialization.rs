//! Port of packages/desktop/src/renderer/initialization.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/renderer/initialization.ts: an
//! initialization error is surfaced before rendering server providers, Electron's
//! remote-invocation wrapper is stripped from the message, falsy errors are not
//! discarded, and a pending (loading) initialization is awaited without reading
//! its data.

#[allow(dead_code)]
mod initialization {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub enum InitError {
        NotImplemented(&'static str),
        Failure {
            message: String,
            local_server_startup: bool,
        },
    }

    impl fmt::Display for InitError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.message())
        }
    }

    impl std::error::Error for InitError {}

    impl InitError {
        pub fn message(&self) -> &str {
            match self {
                InitError::NotImplemented(note) => note,
                InitError::Failure { message, .. } => message,
            }
        }

        pub fn local_server_startup(&self) -> bool {
            match self {
                InitError::NotImplemented(_) => false,
                InitError::Failure {
                    local_server_startup,
                    ..
                } => *local_server_startup,
            }
        }
    }

    pub const NOTE: &str = "porting: desktop renderer initialization not implemented";

    pub struct Invocation<D> {
        pub error: Option<String>,
        pub loading: bool,
        pub produce: Option<Box<dyn FnOnce() -> D>>,
    }

    pub fn initialization_data<D>(_invocation: Invocation<D>) -> Result<D, InitError> {
        Err(InitError::NotImplemented(NOTE))
    }

    pub fn initialization_ready<D>(_invocation: Invocation<D>) -> Result<bool, InitError> {
        Err(InitError::NotImplemented(NOTE))
    }
}

use std::cell::Cell;
use std::rc::Rc;

use initialization::{initialization_data, initialization_ready, Invocation, NOTE};

fn invocation<D>(
    error: Option<&str>,
    loading: bool,
    produce: Option<Box<dyn FnOnce() -> D>>,
) -> Invocation<D> {
    Invocation {
        error: error.map(str::to_string),
        loading,
        produce,
    }
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn throws_the_original_initialization_error_before_rendering_server_providers() {
    let failure = initialization_data(invocation::<String>(
        Some("sidecar startup failed"),
        false,
        None,
    ))
    .expect_err(NOTE);

    assert_eq!(failure.message(), "sidecar startup failed");
    assert!(failure.local_server_startup());
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn removes_electrons_remote_invocation_wrapper_from_startup_errors() {
    let failure = initialization_data(invocation::<String>(
        Some(
            "Error invoking remote method 'await-initialization': Error: Cannot migrate session_message projections",
        ),
        false,
        None,
    ))
    .expect_err(NOTE);

    assert_eq!(
        failure.message(),
        "Cannot migrate session_message projections"
    );
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn returns_initialized_sidecar_data() {
    let data = initialization_data(invocation(
        None,
        false,
        Some(Box::new(|| "http://127.0.0.1:1234".to_string())),
    ))
    .expect(NOTE);
    assert_eq!(data, "http://127.0.0.1:1234");
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn does_not_discard_falsy_initialization_errors() {
    let failure = initialization_data(invocation::<String>(Some(""), false, None)).expect_err(NOTE);
    assert_eq!(failure.message(), "");
    assert!(failure.local_server_startup());
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn checks_initialization_errors_before_rendering_server_providers() {
    let failure = initialization_ready(invocation::<String>(
        Some("sidecar startup failed"),
        false,
        None,
    ))
    .expect_err(NOTE);
    assert_eq!(failure.message(), "sidecar startup failed");
}

#[test]
#[ignore = "porting: desktop renderer initialization not implemented"]
fn waits_for_pending_initialization_without_reading_it() {
    let reads = Rc::new(Cell::new(0usize));
    let counter = reads.clone();
    let pending = initialization_ready(invocation(
        None,
        true,
        Some(Box::new(move || {
            counter.set(counter.get() + 1);
            "unused"
        })),
    ));

    assert!(!pending.expect(NOTE));
    assert_eq!(reads.get(), 0);
}
