//! Port of packages/desktop/src/main/updater-controller.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/updater-controller.ts: a single
//! authoritative ready state is published after check+download; a persisted
//! target is revalidated on launch; an already-installed target is cleared before
//! checking; concurrent checks coalesce; and quitting/install failures return the
//! controller to `ready`.
//! Re-derived: Effect async plumbing is expressed as an owned state machine with
//! synchronous test drivers.

#[allow(dead_code)]
mod updater_controller {
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

    pub const NOTE: &str = "porting: desktop updater controller not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum UpdaterStatus {
        Idle,
        Checking,
        Downloading,
        Ready,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdaterState {
        pub status: UpdaterStatus,
        pub version: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdaterReadyRecord {
        pub version: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct CheckResult {
        pub is_update_available: bool,
        pub version: Option<String>,
    }

    pub type CheckFn = Box<dyn Fn() -> PortResult<CheckResult>>;
    pub type DownloadFn = Box<dyn Fn() -> PortResult<()>>;
    pub type InstallFn = Box<dyn Fn()>;
    pub type ReadyGetFn = Box<dyn Fn() -> Option<UpdaterReadyRecord>>;
    pub type ReadySetFn = Box<dyn Fn(UpdaterReadyRecord)>;
    pub type ReadyClearFn = Box<dyn Fn()>;
    pub type StopFn = Box<dyn Fn() -> PortResult<()>>;

    pub struct UpdaterBackend {
        pub check_for_updates: CheckFn,
        pub download_update: DownloadFn,
        pub quit_and_install: InstallFn,
    }

    pub struct UpdaterPersistence {
        pub get: ReadyGetFn,
        pub set: ReadySetFn,
        pub clear: ReadyClearFn,
    }

    pub struct UpdaterControllerConfig {
        pub enabled: bool,
        pub current_version: String,
        pub backend: UpdaterBackend,
        pub persistence: UpdaterPersistence,
        pub stop: StopFn,
    }

    pub struct UpdaterController;

    impl UpdaterController {
        pub fn subscribe(&self, _listener: Box<dyn FnMut(&UpdaterState)>) {}

        pub fn start(&self) -> PortResult<()> {
            stub()
        }

        pub fn check(&self) -> PortResult<()> {
            stub()
        }

        pub fn install(&self) -> PortResult<()> {
            stub()
        }

        pub fn get_state(&self) -> PortResult<UpdaterState> {
            stub()
        }
    }

    pub fn create_updater_controller(_config: UpdaterControllerConfig) -> UpdaterController {
        UpdaterController
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use updater_controller::{
    create_updater_controller, CheckResult, NotImplemented, UpdaterBackend, UpdaterController,
    UpdaterControllerConfig, UpdaterPersistence, UpdaterReadyRecord, UpdaterState, UpdaterStatus,
    NOTE,
};

struct App {
    controller: UpdaterController,
    calls: Rc<RefCell<Vec<String>>>,
    ready: Rc<RefCell<Option<UpdaterReadyRecord>>>,
}

fn setup(current_version: &str, ready: Option<UpdaterReadyRecord>) -> App {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let ready_state = Rc::new(RefCell::new(ready));

    let check_calls = calls.clone();
    let download_calls = calls.clone();
    let install_calls = calls.clone();
    let backend = UpdaterBackend {
        check_for_updates: Box::new(move || {
            check_calls.borrow_mut().push("check".to_string());
            Ok(CheckResult {
                is_update_available: true,
                version: Some("2.0.0".to_string()),
            })
        }),
        download_update: Box::new(move || {
            download_calls.borrow_mut().push("download".to_string());
            Ok(())
        }),
        quit_and_install: Box::new(move || {
            install_calls.borrow_mut().push("install".to_string());
        }),
    };

    let get_ready = ready_state.clone();
    let set_ready = ready_state.clone();
    let clear_ready = ready_state.clone();
    let persistence = UpdaterPersistence {
        get: Box::new(move || get_ready.borrow().clone()),
        set: Box::new(move |value: UpdaterReadyRecord| {
            *set_ready.borrow_mut() = Some(value);
        }),
        clear: Box::new(move || {
            *clear_ready.borrow_mut() = None;
        }),
    };

    let stop_calls = calls.clone();
    let controller = create_updater_controller(UpdaterControllerConfig {
        enabled: true,
        current_version: current_version.to_string(),
        backend,
        persistence,
        stop: Box::new(move || {
            stop_calls.borrow_mut().push("stop".to_string());
            Ok(())
        }),
    });

    App {
        controller,
        calls,
        ready: ready_state,
    }
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn checks_downloads_persists_and_publishes_one_authoritative_ready_state() {
    let app = setup("1.0.0", None);
    let states = Rc::new(RefCell::new(Vec::new()));
    let states_sink = states.clone();
    app.controller
        .subscribe(Box::new(move |state: &UpdaterState| {
            states_sink.borrow_mut().push(state.clone());
        }));

    app.controller.start().expect(NOTE);

    assert_eq!(*app.calls.borrow(), vec!["check", "download"]);
    assert_eq!(
        *app.ready.borrow(),
        Some(UpdaterReadyRecord {
            version: "2.0.0".to_string()
        })
    );
    let statuses: Vec<UpdaterStatus> = states
        .borrow()
        .iter()
        .map(|state| state.status.clone())
        .collect();
    assert_eq!(
        statuses,
        vec![
            UpdaterStatus::Idle,
            UpdaterStatus::Checking,
            UpdaterStatus::Downloading,
            UpdaterStatus::Ready
        ]
    );
    assert_eq!(
        app.controller.get_state().expect(NOTE),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn revalidates_a_persisted_target_through_the_updater_cache_on_launch() {
    let app = setup(
        "1.0.0",
        Some(UpdaterReadyRecord {
            version: "2.0.0".to_string(),
        }),
    );

    app.controller.start().expect(NOTE);

    assert_eq!(*app.calls.borrow(), vec!["check", "download"]);
    assert_eq!(
        app.controller.get_state().expect(NOTE),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn clears_a_target_already_installed_before_checking() {
    let app = setup(
        "2.0.0",
        Some(UpdaterReadyRecord {
            version: "2.0.0".to_string(),
        }),
    );

    app.controller.start().expect(NOTE);

    assert_eq!(*app.ready.borrow(), None);
    assert_eq!(*app.calls.borrow(), vec!["check"]);
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn coalesces_concurrent_checks() {
    let app = setup("1.0.0", None);

    let _ = (
        app.controller.check(),
        app.controller.check(),
        app.controller.check(),
    );

    assert_eq!(*app.calls.borrow(), vec!["check", "download"]);
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn returns_to_ready_when_quit_and_install_returns_without_exiting() {
    let app = setup("1.0.0", None);
    app.controller.start().expect(NOTE);

    app.controller.install().expect(NOTE);

    assert_eq!(
        *app.calls.borrow(),
        vec!["check", "download", "stop", "install"]
    );
    assert_eq!(
        app.controller.get_state().expect(NOTE),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
#[ignore = "porting: desktop updater controller not implemented"]
fn returns_to_ready_when_installation_cannot_start() {
    let failed = create_updater_controller(UpdaterControllerConfig {
        enabled: true,
        current_version: "1.0.0".to_string(),
        backend: UpdaterBackend {
            check_for_updates: Box::new(|| {
                Ok(CheckResult {
                    is_update_available: true,
                    version: Some("2.0.0".to_string()),
                })
            }),
            download_update: Box::new(|| Ok(())),
            quit_and_install: Box::new(|| {}),
        },
        persistence: UpdaterPersistence {
            get: Box::new(|| None),
            set: Box::new(|_value| {}),
            clear: Box::new(|| {}),
        },
        stop: Box::new(|| Err(NotImplemented("stop failed"))),
    });

    failed.start().expect(NOTE);
    let error = failed.install().expect_err(NOTE);
    assert!(error.to_string().contains("stop failed"));
    assert_eq!(
        failed.get_state().expect(NOTE),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}
