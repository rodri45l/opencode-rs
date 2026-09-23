//! System context sources and registry.
//!
//! Ports the observable behaviour of `packages/core/src/system-context/*`: a
//! system context is an ordered set of keyed sources that initialize a baseline
//! plus a structured snapshot, reconcile updates only when a value changes, and
//! block replacement while an admitted source is unavailable. The registry
//! registers scoped entries and loads them in stable key order.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A namespaced system-context key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(String);

impl Key {
    /// Validate and construct a namespaced key.
    pub fn make(_value: &str) -> CoreResult<Self> {
        Err(CoreError::NotImplemented("system_context::Key::make"))
    }

    /// Borrow the key string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A loaded source value.
#[derive(Debug, Clone, PartialEq)]
pub enum Loaded {
    /// A canonical JSON value.
    Value(Value),
    /// The source is temporarily unavailable.
    Unavailable,
}

/// A baseline renderer.
pub type Baseline = fn(&Value) -> String;
/// An update renderer.
pub type Update = fn(&Value, &Value) -> String;
/// A removal renderer.
pub type Removed = fn(&Value) -> String;

/// A single system-context source.
pub struct Source {
    /// Source key.
    pub key: Key,
    /// Loaded value.
    pub load: Loaded,
    /// Baseline renderer.
    pub baseline: Baseline,
    /// Update renderer.
    pub update: Update,
    /// Removal renderer, when the source can be removed.
    pub removed: Option<Removed>,
}

impl Source {
    /// Construct a source.
    pub fn new(
        key: Key,
        load: Loaded,
        baseline: Baseline,
        update: Update,
        removed: Option<Removed>,
    ) -> Self {
        Self {
            key,
            load,
            baseline,
            update,
            removed,
        }
    }
}

/// A stored snapshot entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotEntry {
    /// Stored value.
    pub value: Value,
    /// Removal message, when any.
    pub removed: Option<String>,
}

/// A structured snapshot.
pub type Snapshot = BTreeMap<String, SnapshotEntry>;

/// An initialized system context.
#[derive(Debug, Clone, PartialEq)]
pub struct Initialized {
    /// Rendered baseline.
    pub baseline: String,
    /// Structured snapshot.
    pub snapshot: Snapshot,
}

/// A coherent source observation used for replacement.
#[derive(Debug, Clone, PartialEq)]
pub struct Generation {
    /// Rendered baseline.
    pub baseline: String,
}

/// The outcome of reconciling a context against a previous snapshot.
#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileResult {
    /// The context changed; render `text` and store `snapshot`.
    Updated {
        /// Rendered update text.
        text: String,
        /// New snapshot.
        snapshot: Snapshot,
    },
    /// Nothing changed.
    Unchanged,
    /// A coherent replacement generation is ready.
    ReplacementReady {
        /// The replacement generation.
        generation: Generation,
    },
    /// Replacement is blocked by an unavailable admitted source.
    ReplacementBlocked,
}

/// An ordered set of system-context sources.
pub struct SystemContext {
    sources: Vec<Source>,
}

impl SystemContext {
    /// An empty context.
    pub fn empty() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// A context with one source.
    pub fn make(source: Source) -> Self {
        Self {
            sources: vec![source],
        }
    }

    /// Combine sources, rejecting duplicate source keys.
    pub fn combine(sources: Vec<Source>) -> CoreResult<Self> {
        Ok(Self { sources })
    }

    /// Initialize the baseline and snapshot.
    pub fn initialize(&self) -> CoreResult<Initialized> {
        let _ = &self.sources;
        Err(CoreError::NotImplemented(
            "system_context::SystemContext::initialize",
        ))
    }

    /// Reconcile against a previous snapshot.
    pub fn reconcile(&self, _previous: &Snapshot) -> CoreResult<ReconcileResult> {
        Err(CoreError::NotImplemented(
            "system_context::SystemContext::reconcile",
        ))
    }

    /// Replace from a coherent observation.
    pub fn replace(&self, _previous: &Snapshot) -> CoreResult<ReconcileResult> {
        Err(CoreError::NotImplemented(
            "system_context::SystemContext::replace",
        ))
    }
}

/// A registry entry that produces a source on demand.
pub struct RegistryEntry {
    /// Entry key.
    pub key: Key,
    /// Producer for the entry's source.
    pub load: Box<dyn Fn() -> CoreResult<Loaded>>,
}

/// Scoped system-context entry registry.
#[derive(Default)]
pub struct SystemContextRegistry {
    entries: Vec<RegistryEntry>,
}

impl SystemContextRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an entry, rejecting duplicate entry keys.
    pub fn register(&mut self, _entry: RegistryEntry) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "system_context::SystemContextRegistry::register",
        ))
    }

    /// Load the registered entries into a combined context.
    pub fn load(&self) -> CoreResult<SystemContext> {
        let _ = &self.entries;
        Err(CoreError::NotImplemented(
            "system_context::SystemContextRegistry::load",
        ))
    }
}

/// Built-in environment and date context input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinEnv {
    /// Working directory.
    pub working_directory: String,
    /// Workspace root.
    pub workspace_root: String,
    /// Whether the directory is a git repo.
    pub is_git_repo: bool,
    /// Platform name.
    pub platform: String,
    /// Host-local date string.
    pub date: String,
}

/// Built-in system context.
#[derive(Debug, Default)]
pub struct SystemContextBuiltIns;

impl SystemContextBuiltIns {
    /// Render the built-in environment and date baseline.
    pub fn render(_env: &BuiltinEnv) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "system_context::SystemContextBuiltIns::render",
        ))
    }
}
