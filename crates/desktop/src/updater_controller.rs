//! Updater controller.
//!
//! Port of `packages/desktop/src/main/updater-controller.ts` (upstream 18ef3cc):
//! a single authoritative ready state is published after check+download, a
//! persisted target is revalidated on launch, an already-installed target is
//! cleared before checking, concurrent checks coalesce, and install failures
//! return the controller to `ready`. Effect async plumbing is re-derived as a
//! synchronous state machine.

use std::cell::RefCell;

pub use crate::error::NotImplemented;

/// The updater status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdaterStatus {
    Idle,
    Checking,
    Downloading,
    Ready,
}

/// The published updater state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterState {
    pub status: UpdaterStatus,
    pub version: Option<String>,
}

/// A persisted ready record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdaterReadyRecord {
    pub version: String,
}

/// The result of an update check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub is_update_available: bool,
    pub version: Option<String>,
}

pub type CheckFn = Box<dyn Fn() -> Result<CheckResult, NotImplemented>>;
pub type DownloadFn = Box<dyn Fn() -> Result<(), NotImplemented>>;
pub type InstallFn = Box<dyn Fn()>;
pub type ReadyGetFn = Box<dyn Fn() -> Option<UpdaterReadyRecord>>;
pub type ReadySetFn = Box<dyn Fn(UpdaterReadyRecord)>;
pub type ReadyClearFn = Box<dyn Fn()>;
pub type StopFn = Box<dyn Fn() -> Result<(), NotImplemented>>;

/// The updater backend operations.
pub struct UpdaterBackend {
    pub check_for_updates: CheckFn,
    pub download_update: DownloadFn,
    pub quit_and_install: InstallFn,
}

/// The updater persistence operations.
pub struct UpdaterPersistence {
    pub get: ReadyGetFn,
    pub set: ReadySetFn,
    pub clear: ReadyClearFn,
}

/// The controller configuration.
pub struct UpdaterControllerConfig {
    pub enabled: bool,
    pub current_version: String,
    pub backend: UpdaterBackend,
    pub persistence: UpdaterPersistence,
    pub stop: StopFn,
}

type Listener = Box<dyn FnMut(&UpdaterState)>;

/// A synchronous updater controller.
pub struct UpdaterController {
    enabled: bool,
    current_version: String,
    backend: UpdaterBackend,
    persistence: UpdaterPersistence,
    stop: StopFn,
    state: RefCell<UpdaterState>,
    listeners: RefCell<Vec<Listener>>,
}

impl UpdaterController {
    fn transition(&self, status: UpdaterStatus, version: Option<String>) {
        let next = UpdaterState { status, version };
        *self.state.borrow_mut() = next.clone();
        for listener in self.listeners.borrow_mut().iter_mut() {
            listener(&next);
        }
    }

    /// Subscribe to state changes; the listener is invoked immediately.
    pub fn subscribe(&self, listener: Box<dyn FnMut(&UpdaterState)>) {
        let mut listeners = self.listeners.borrow_mut();
        listeners.push(listener);
        let state = self.state.borrow().clone();
        if let Some(listener) = listeners.last_mut() {
            listener(&state);
        }
    }

    /// Revalidate a persisted target, then check for an update.
    pub fn start(&self) -> Result<(), NotImplemented> {
        let ready = (self.persistence.get)();
        if ready
            .map(|record| record.version == self.current_version)
            .unwrap_or(false)
        {
            (self.persistence.clear)();
        }
        self.check()
    }

    /// Check, download, persist, and publish a ready state.
    pub fn check(&self) -> Result<(), NotImplemented> {
        if !self.enabled {
            return Ok(());
        }
        if self.state.borrow().status == UpdaterStatus::Ready {
            return Ok(());
        }

        self.transition(UpdaterStatus::Checking, None);
        let result = (self.backend.check_for_updates)()?;
        let version = result.version.clone();
        let usable = result.is_update_available
            && version.is_some()
            && version.as_deref() != Some(self.current_version.as_str());
        if !usable {
            (self.persistence.clear)();
            self.transition(UpdaterStatus::Idle, None);
            return Ok(());
        }

        let version = version.expect("checked version");
        self.transition(UpdaterStatus::Downloading, Some(version.clone()));
        (self.backend.download_update)()?;
        (self.persistence.set)(UpdaterReadyRecord {
            version: version.clone(),
        });
        self.transition(UpdaterStatus::Ready, Some(version));
        Ok(())
    }

    /// Stop and install, returning to `ready` if the app does not exit.
    pub fn install(&self) -> Result<(), NotImplemented> {
        if self.state.borrow().status != UpdaterStatus::Ready {
            return Err(NotImplemented("Update is not ready to install".to_string()));
        }
        let version = self.state.borrow().version.clone();
        match (self.stop)() {
            Ok(()) => {
                (self.backend.quit_and_install)();
                self.transition(UpdaterStatus::Ready, version);
                Ok(())
            }
            Err(error) => {
                self.transition(UpdaterStatus::Ready, version);
                Err(error)
            }
        }
    }

    /// The current state.
    pub fn get_state(&self) -> UpdaterState {
        self.state.borrow().clone()
    }
}

/// Create an updater controller.
pub fn create_updater_controller(config: UpdaterControllerConfig) -> UpdaterController {
    UpdaterController {
        enabled: config.enabled,
        current_version: config.current_version,
        backend: config.backend,
        persistence: config.persistence,
        stop: config.stop,
        state: RefCell::new(UpdaterState {
            status: UpdaterStatus::Idle,
            version: None,
        }),
        listeners: RefCell::new(Vec::new()),
    }
}
