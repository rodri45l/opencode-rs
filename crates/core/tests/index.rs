//! Port of packages/core/test/system-context/index.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a system context is an ordered set of namespaced sources
//! that initialize a baseline plus a structured snapshot, reconcile updates only
//! when a value changes, use a baseline for newly added sources, retain admitted
//! snapshots while a source is unavailable, block initialization and replacement
//! while a source is unavailable, and render removals in stable key order.
//! Re-derived against the Rust API; the `Effect`/`Deferred` mechanics are dropped.

use std::collections::BTreeMap;

use opencode_core::system_context::{
    Key, Loaded, ReconcileResult, Snapshot, SnapshotEntry, Source, SystemContext,
};
use serde_json::{json, Value};

fn identity(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn baseline_value(value: &Value) -> String {
    format!("Directory: {}", value.as_str().unwrap_or_default())
}

fn update_value(_previous: &Value, current: &Value) -> String {
    current.as_str().unwrap_or_default().to_string()
}

fn removed_value(_value: &Value) -> String {
    "Removed".to_string()
}

fn source(
    key: &str,
    value: Loaded,
    baseline: fn(&Value) -> String,
    update: fn(&Value, &Value) -> String,
    removed: Option<fn(&Value) -> String>,
) -> Source {
    Source::new(
        Key::make(key).expect("valid key"),
        value,
        baseline,
        update,
        removed,
    )
}

fn string_source(key: &str, value: &str) -> Source {
    source(
        key,
        Loaded::Value(json!(value)),
        identity,
        update_value,
        None,
    )
}

fn entry(value: &str, removed: Option<&str>) -> SnapshotEntry {
    SnapshotEntry {
        value: json!(value),
        removed: removed.map(str::to_string),
    }
}

#[test]
fn requires_namespaced_source_keys() {
    assert!(Key::make("core/date").is_ok());
    assert!(Key::make("date").is_err());
}

#[test]
fn combines_contexts_in_order() {
    let context = SystemContext::combine(vec![
        string_source("core/date", "date"),
        string_source("core/location", "location"),
    ])
    .unwrap();

    assert_eq!(context.initialize().unwrap().baseline, "date\n\nlocation");
}

#[test]
fn rejects_duplicate_source_keys() {
    let result = SystemContext::combine(vec![
        string_source("core/date", "one"),
        string_source("core/date", "two"),
    ]);
    assert!(result.is_err());
}

#[test]
fn initializes_a_baseline_with_a_structured_snapshot() {
    let context = SystemContext::combine(vec![
        source(
            "core/date",
            Loaded::Value(json!("2026-06-03")),
            baseline_value,
            update_value,
            Some(removed_value),
        ),
        source(
            "core/location",
            Loaded::Value(json!("/repo")),
            baseline_value,
            update_value,
            None,
        ),
    ])
    .unwrap();

    let initialized = context.initialize().unwrap();
    assert_eq!(
        initialized.baseline,
        "Directory: 2026-06-03\n\nDirectory: /repo"
    );
    assert_eq!(initialized.snapshot["core/date"].value, json!("2026-06-03"));
    assert_eq!(initialized.snapshot["core/location"].value, json!("/repo"));
}

#[test]
fn renders_updates_only_after_a_structured_value_changes() {
    let changed = SystemContext::combine(vec![
        string_source("core/date", "2026-06-04"),
        string_source("core/location", "/repo"),
    ])
    .unwrap();

    let previous: Snapshot = BTreeMap::from([
        ("core/date".to_string(), entry("2026-06-03", None)),
        ("core/location".to_string(), entry("/repo", None)),
    ]);

    let result = changed.reconcile(&previous).unwrap();
    assert!(matches!(result, ReconcileResult::Updated { .. }));

    let unchanged = SystemContext::combine(vec![
        string_source("core/date", "2026-06-03"),
        string_source("core/location", "/repo"),
    ])
    .unwrap();
    assert_eq!(
        unchanged.reconcile(&previous).unwrap(),
        ReconcileResult::Unchanged
    );
}

#[test]
fn uses_the_baseline_for_a_newly_added_source() {
    let context = source(
        "core/skills",
        Loaded::Value(json!("effect")),
        |value| format!("Available skill: {}", value.as_str().unwrap_or_default()),
        update_value,
        None,
    );

    assert_eq!(
        SystemContext::make(context)
            .reconcile(&Snapshot::new())
            .unwrap(),
        ReconcileResult::Updated {
            text: "Available skill: effect".to_string(),
            snapshot: BTreeMap::from([("core/skills".to_string(), entry("effect", None))]),
        }
    );
}

#[test]
fn retains_admitted_snapshots_while_a_source_is_unavailable() {
    let previous: Snapshot = BTreeMap::from([(
        "core/remote".to_string(),
        entry("instructions", Some("Instructions removed")),
    )]);
    let context = source(
        "core/remote",
        Loaded::Unavailable,
        identity,
        update_value,
        None,
    );
    let context = SystemContext::make(context);

    assert_eq!(
        context.reconcile(&previous).unwrap(),
        ReconcileResult::Unchanged
    );
    assert_eq!(
        context.replace(&previous).unwrap(),
        ReconcileResult::ReplacementBlocked
    );
    assert!(matches!(
        context.replace(&Snapshot::new()).unwrap(),
        ReconcileResult::ReplacementReady { .. }
    ));
}

#[test]
fn blocks_initialization_while_a_source_is_unavailable() {
    let context = source(
        "core/remote",
        Loaded::Unavailable,
        identity,
        update_value,
        None,
    );
    assert!(SystemContext::make(context).initialize().is_err());
}

#[test]
fn emits_the_previously_stored_removal_message() {
    let previous: Snapshot = BTreeMap::from([(
        "core/instructions".to_string(),
        entry(
            "contents",
            Some("Instructions removed; stop applying them."),
        ),
    )]);

    assert_eq!(
        SystemContext::empty().reconcile(&previous).unwrap(),
        ReconcileResult::Updated {
            text: "Instructions removed; stop applying them.".to_string(),
            snapshot: Snapshot::new(),
        }
    );
}

#[test]
fn renders_multiple_removals_in_stable_key_order() {
    let previous: Snapshot = BTreeMap::from([
        ("core/z".to_string(), entry("z", Some("Removed z"))),
        ("core/a".to_string(), entry("a", Some("Removed a"))),
    ]);

    match SystemContext::empty().reconcile(&previous).unwrap() {
        ReconcileResult::Updated { text, .. } => assert_eq!(text, "Removed a\n\nRemoved z"),
        other => panic!("expected an update, got {other:?}"),
    }
}
