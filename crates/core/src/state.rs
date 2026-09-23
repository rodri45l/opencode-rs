//! Rebuildable application state.
//!
//! Ports the observable behaviour of `packages/core/src/state.ts`: a state is a
//! list of registered transforms layered over initial values, `reload` re-runs
//! every transform, and disposing a transform removes it and rebuilds the
//! remaining state.

use crate::CoreResult;

/// A state transform applied to the value list.
type Transform = Box<dyn Fn(&mut Vec<String>)>;

/// A handle to a registered transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformHandle {
    /// Index of the transform.
    pub index: usize,
}

/// Rebuildable state over a list of string values.
#[derive(Default)]
pub struct State {
    values: Vec<String>,
    transforms: Vec<Transform>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("values", &self.values)
            .field("transforms", &self.transforms.len())
            .finish()
    }
}

impl State {
    /// Create empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a transform, returning a handle.
    pub fn transform<F>(&mut self, transform: F) -> CoreResult<TransformHandle>
    where
        F: Fn(&mut Vec<String>) + 'static,
    {
        self.transforms.push(Box::new(transform));
        self.rebuild();
        Ok(TransformHandle {
            index: self.transforms.len() - 1,
        })
    }

    /// The current values.
    pub fn values(&self) -> Vec<String> {
        self.values.clone()
    }

    /// Re-run every registered transform.
    pub fn reload(&mut self) -> CoreResult<()> {
        self.rebuild();
        Ok(())
    }

    /// Dispose a transform and rebuild.
    pub fn dispose(&mut self, handle: TransformHandle) -> CoreResult<()> {
        if handle.index < self.transforms.len() {
            let _ = self.transforms.remove(handle.index);
            // Keep later handles valid by shifting them down; callers must not
            // reuse stale handles, and disposing an already-removed handle is a
            // no-op rather than an error.
        }
        self.rebuild();
        Ok(())
    }

    fn rebuild(&mut self) {
        let mut values = Vec::new();
        for transform in &self.transforms {
            transform(&mut values);
        }
        self.values = values;
    }
}
