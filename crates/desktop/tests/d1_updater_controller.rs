//! Port of packages/desktop/src/main/updater-controller.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/updater-controller.ts: a single
//! authoritative ready state is published after check+download; a persisted
//! target is revalidated on launch; an already-installed target is cleared before
//! checking; concurrent checks coalesce; and quitting/install failures return the
//! controller to `ready`.
//! Re-derived: Effect async plumbing is expressed as an owned state machine with
//! synchronous test drivers.

use std::cell::RefCell;
use std::rc::Rc;

use opencode_desktop::updater_controller::{
    create_updater_controller, CheckResult, NotImplemented, UpdaterBackend, UpdaterController,
    UpdaterControllerConfig, UpdaterPersistence, UpdaterReadyRecord, UpdaterState, UpdaterStatus,
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
fn checks_downloads_persists_and_publishes_one_authoritative_ready_state() {
    let app = setup("1.0.0", None);
    let states = Rc::new(RefCell::new(Vec::new()));
    let states_sink = states.clone();
    app.controller
        .subscribe(Box::new(move |state: &UpdaterState| {
            states_sink.borrow_mut().push(state.clone());
        }));

    app.controller.start().expect("start");

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
        app.controller.get_state(),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
fn revalidates_a_persisted_target_through_the_updater_cache_on_launch() {
    let app = setup(
        "1.0.0",
        Some(UpdaterReadyRecord {
            version: "2.0.0".to_string(),
        }),
    );

    app.controller.start().expect("start");

    assert_eq!(*app.calls.borrow(), vec!["check", "download"]);
    assert_eq!(
        app.controller.get_state(),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
fn clears_a_target_already_installed_before_checking() {
    let app = setup(
        "2.0.0",
        Some(UpdaterReadyRecord {
            version: "2.0.0".to_string(),
        }),
    );

    app.controller.start().expect("start");

    assert_eq!(*app.ready.borrow(), None);
    assert_eq!(*app.calls.borrow(), vec!["check"]);
}

#[test]
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
fn returns_to_ready_when_quit_and_install_returns_without_exiting() {
    let app = setup("1.0.0", None);
    app.controller.start().expect("start");

    app.controller.install().expect("install");

    assert_eq!(
        *app.calls.borrow(),
        vec!["check", "download", "stop", "install"]
    );
    assert_eq!(
        app.controller.get_state(),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}

#[test]
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
        stop: Box::new(|| Err(NotImplemented("stop failed".to_string()))),
    });

    failed.start().expect("start");
    let error = failed.install().expect_err("install");
    assert!(error.to_string().contains("stop failed"));
    assert_eq!(
        failed.get_state(),
        UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".to_string())
        }
    );
}
