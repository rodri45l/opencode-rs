//! Window registry.
//!
//! Port of `packages/desktop/src/main/window-registry.ts` (upstream 18ef3cc):
//! persisted ids are restored (malformed entries dropped), registration persists
//! each id once, and a deliberate close forgets a window unless it was the last
//! one or the app is quitting.

/// A persisted window id slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedId {
    Id(String),
    Invalid,
}

type ReadFn = Box<dyn Fn() -> Vec<PersistedId>>;
type WriteFn = Box<dyn Fn(&[String])>;
type CleanupFn = Box<dyn Fn(&str)>;

/// A tracked window value.
pub struct WindowRegistry<T> {
    windows: Vec<(String, T)>,
    quitting: bool,
    last_focused_id: Option<String>,
    read: ReadFn,
    write: WriteFn,
    cleanup: CleanupFn,
}

impl<T: Clone> WindowRegistry<T> {
    /// The persisted, non-empty ids.
    pub fn persisted(&self) -> Vec<String> {
        (self.read)()
            .into_iter()
            .filter_map(|entry| match entry {
                PersistedId::Id(id) if !id.is_empty() => Some(id),
                _ => None,
            })
            .collect()
    }

    /// Register a window, persisting its id once.
    pub fn register(&mut self, id: &str, window: T) {
        match self.windows.iter_mut().find(|(key, _)| key == id) {
            Some(slot) => slot.1 = window,
            None => self.windows.push((id.to_string(), window)),
        }
        let mut ids = self.persisted();
        if !ids.iter().any(|item| item == id) {
            ids.push(id.to_string());
            (self.write)(&ids);
        }
    }

    /// Mark the app as quitting.
    pub fn set_quitting(&mut self) {
        self.quitting = true;
    }

    /// Set the quitting flag explicitly.
    pub fn set_quitting_flag(&mut self, value: bool) {
        self.quitting = value;
    }

    /// Record the focused window.
    pub fn focused(&mut self, id: &str) {
        self.last_focused_id = Some(id.to_string());
    }

    /// The last focused window, if any.
    pub fn last_focused(&self) -> Option<T> {
        let id = self.last_focused_id.as_ref()?;
        self.windows
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, window)| window.clone())
    }

    /// Handle a window close.
    pub fn closed(&mut self, id: &str) {
        self.windows.retain(|(key, _)| key != id);
        if self.last_focused_id.as_deref() == Some(id) {
            self.last_focused_id = self.windows.first().map(|(key, _)| key.clone());
        }
        if self.quitting || self.windows.is_empty() {
            return;
        }
        let remaining: Vec<String> = self
            .persisted()
            .into_iter()
            .filter(|item| item != id)
            .collect();
        (self.write)(&remaining);
        (self.cleanup)(id);
    }
}

/// Create a window registry backed by the supplied persistence closures.
pub fn create_window_registry<T>(
    read: impl Fn() -> Vec<PersistedId> + 'static,
    write: impl Fn(&[String]) + 'static,
    cleanup: impl Fn(&str) + 'static,
) -> WindowRegistry<T> {
    WindowRegistry {
        windows: Vec::new(),
        quitting: false,
        last_focused_id: None,
        read: Box::new(read),
        write: Box::new(write),
        cleanup: Box::new(cleanup),
    }
}
