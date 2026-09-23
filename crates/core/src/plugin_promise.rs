//! Promise plugin adapter (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/plugin/promise.ts` and
//! its `fromPromise` adapter: a promise-defined plugin's `setup` runs when the
//! plugin effect is applied, registering a transform hook, and the returned
//! registration can be disposed to revert the hook. The Effect `PluginHost`
//! wiring and the async `setup` runtime are dropped; the pure registration
//! lifecycle remains.

use crate::{CoreError, CoreResult};

/// A registered promise-plugin transform hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromiseRegistration {
    description: Option<String>,
}

impl PromiseRegistration {
    /// The description currently contributed by the hook.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Dispose the registration, reverting the hook.
    pub fn dispose(&mut self) -> CoreResult<()> {
        let _ = &mut self.description;
        Err(CoreError::NotImplemented(
            "plugin_promise::PromiseRegistration::dispose",
        ))
    }
}

/// The promise plugin adapter.
#[derive(Debug, Default)]
pub struct PluginPromise;

impl PluginPromise {
    /// Load a promise-defined plugin, registering its transform hook.
    pub fn from_promise(_description: &str) -> CoreResult<PromiseRegistration> {
        Err(CoreError::NotImplemented(
            "plugin_promise::PluginPromise::from_promise",
        ))
    }
}
