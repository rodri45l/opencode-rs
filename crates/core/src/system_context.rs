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
    pub fn make(value: &str) -> CoreResult<Self> {
        let valid = value
            .split_once('/')
            .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty());
        if !valid {
            return Err(CoreError::Invalid(format!(
                "system-context key must be namespaced: {value}"
            )));
        }
        Ok(Self(value.to_string()))
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
        let mut seen = Vec::new();
        for source in &sources {
            if seen.contains(&source.key) {
                return Err(CoreError::Invalid(format!(
                    "duplicate system-context source key: {}",
                    source.key.as_str()
                )));
            }
            seen.push(source.key.clone());
        }
        Ok(Self { sources })
    }

    /// Initialize the baseline and snapshot.
    pub fn initialize(&self) -> CoreResult<Initialized> {
        if self
            .sources
            .iter()
            .any(|source| matches!(source.load, Loaded::Unavailable))
        {
            return Err(CoreError::Message(
                "cannot initialize while a source is unavailable".into(),
            ));
        }
        let baseline = self
            .sources
            .iter()
            .map(|source| match &source.load {
                Loaded::Value(value) => (source.baseline)(value),
                Loaded::Unavailable => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let mut snapshot = Snapshot::new();
        for source in &self.sources {
            if let Loaded::Value(value) = &source.load {
                snapshot.insert(
                    source.key.as_str().to_string(),
                    SnapshotEntry {
                        value: value.clone(),
                        removed: None,
                    },
                );
            }
        }
        Ok(Initialized { baseline, snapshot })
    }

    /// Reconcile against a previous snapshot.
    pub fn reconcile(&self, previous: &Snapshot) -> CoreResult<ReconcileResult> {
        if self
            .sources
            .iter()
            .any(|source| matches!(source.load, Loaded::Unavailable))
        {
            return Ok(ReconcileResult::Unchanged);
        }

        let mut segments: Vec<String> = Vec::new();
        let mut snapshot = Snapshot::new();
        let mut changed = false;

        for source in &self.sources {
            if let Loaded::Value(value) = &source.load {
                let key = source.key.as_str().to_string();
                match previous.get(&key) {
                    Some(entry) => {
                        if entry.value != *value {
                            segments.push((source.update)(&entry.value, value));
                            changed = true;
                        }
                    }
                    None => {
                        segments.push((source.baseline)(value));
                        changed = true;
                    }
                }
                snapshot.insert(
                    key,
                    SnapshotEntry {
                        value: value.clone(),
                        removed: None,
                    },
                );
            }
        }

        let current_keys: Vec<&String> = self.sources.iter().map(|source| &source.key.0).collect();
        for (key, entry) in previous {
            if !current_keys.contains(&key) {
                if let Some(removed) = &entry.removed {
                    segments.push(removed.clone());
                    changed = true;
                }
            }
        }

        if changed {
            Ok(ReconcileResult::Updated {
                text: segments.join("\n\n"),
                snapshot,
            })
        } else {
            Ok(ReconcileResult::Unchanged)
        }
    }

    /// Replace from a coherent observation.
    pub fn replace(&self, previous: &Snapshot) -> CoreResult<ReconcileResult> {
        let has_unavailable = self
            .sources
            .iter()
            .any(|source| matches!(source.load, Loaded::Unavailable));
        if has_unavailable {
            let admitted = self
                .sources
                .iter()
                .any(|source| previous.contains_key(source.key.as_str()));
            if admitted {
                return Ok(ReconcileResult::ReplacementBlocked);
            }
            // Nothing was admitted for the unavailable sources, so a coherent
            // generation from the available sources can still be produced.
            let baseline = self
                .sources
                .iter()
                .filter_map(|source| match &source.load {
                    Loaded::Value(value) => Some((source.baseline)(value)),
                    Loaded::Unavailable => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            return Ok(ReconcileResult::ReplacementReady {
                generation: Generation { baseline },
            });
        }
        let initialized = self.initialize()?;
        Ok(ReconcileResult::ReplacementReady {
            generation: Generation {
                baseline: initialized.baseline,
            },
        })
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
    pub fn register(&mut self, entry: RegistryEntry) -> CoreResult<()> {
        if self
            .entries
            .iter()
            .any(|existing| existing.key == entry.key)
        {
            return Err(CoreError::Invalid(format!(
                "duplicate registry entry key: {}",
                entry.key.as_str()
            )));
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Load the registered entries into a combined context.
    pub fn load(&self) -> CoreResult<SystemContext> {
        let mut entries: Vec<&RegistryEntry> = self.entries.iter().collect();
        entries.sort_by(|left, right| left.key.cmp(&right.key));
        let mut sources = Vec::new();
        let mut seen = Vec::new();
        for entry in entries {
            let loaded = (entry.load)()?;
            if seen.contains(&entry.key) {
                return Err(CoreError::Invalid(format!(
                    "duplicate source key: {}",
                    entry.key.as_str()
                )));
            }
            seen.push(entry.key.clone());
            sources.push(Source::new(
                entry.key.clone(),
                loaded,
                identity_baseline,
                identity_update,
                None,
            ));
        }
        SystemContext::combine(sources)
    }
}

fn identity_baseline(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn identity_update(_previous: &Value, current: &Value) -> String {
    current.as_str().unwrap_or_default().to_string()
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
    pub fn render(env: &BuiltinEnv) -> CoreResult<String> {
        Ok([
            "Here is some useful information about the environment you are running in:".to_string(),
            "<env>".to_string(),
            format!("  Working directory: {}", env.working_directory),
            format!("  Workspace root folder: {}", env.workspace_root),
            format!(
                "  Is directory a git repo: {}",
                if env.is_git_repo { "yes" } else { "no" }
            ),
            format!("  Platform: {}", env.platform),
            "</env>".to_string(),
            String::new(),
            format!("Today's date: {}", env.date),
        ]
        .join("\n"))
    }
}
