//! Port of packages/core/test/system-context/registry.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the registry loads an empty context when there are no
//! entries, loads scoped entries in stable key order, re-evaluates producers on
//! each load, and rejects duplicate entry keys and duplicate source keys from
//! separate entries. Re-derived against the Rust API; scope-removal is dropped.

use std::cell::Cell;
use std::rc::Rc;

use opencode_core::system_context::{Key, Loaded, RegistryEntry, SystemContextRegistry};
use serde_json::json;

const NOTE: &str = "porting: system context registry not implemented";

fn entry(key: &str, value: &'static str) -> RegistryEntry {
    RegistryEntry {
        key: Key::make(key).expect("valid key"),
        load: Box::new(move || Ok(Loaded::Value(json!(value)))),
    }
}

#[test]
#[ignore = "porting: system context registry not implemented"]
fn loads_empty_system_context_when_there_are_no_entries() {
    let registry = SystemContextRegistry::new();
    let context = registry.load().expect(NOTE);
    let initialized = context.initialize().expect(NOTE);

    assert_eq!(initialized.baseline, "");
    assert!(initialized.snapshot.is_empty());
}

#[test]
#[ignore = "porting: system context registry not implemented"]
fn loads_scoped_entries_in_stable_key_order() {
    let mut registry = SystemContextRegistry::new();
    registry
        .register(entry("test/second", "second"))
        .expect(NOTE);
    registry.register(entry("test/first", "first")).expect(NOTE);

    let context = registry.load().expect(NOTE);
    assert_eq!(
        context.initialize().expect(NOTE).baseline,
        "first\n\nsecond"
    );
}

#[test]
#[ignore = "porting: system context registry not implemented"]
fn re_evaluates_entry_producers_on_each_load() {
    let loads = Rc::new(Cell::new(0));
    let counter = Rc::clone(&loads);
    let mut registry = SystemContextRegistry::new();
    registry
        .register(RegistryEntry {
            key: Key::make("test/dynamic").expect("valid key"),
            load: Box::new(move || {
                counter.set(counter.get() + 1);
                Ok(Loaded::Value(json!("value")))
            }),
        })
        .expect(NOTE);

    registry.load().expect(NOTE);
    registry.load().expect(NOTE);
    assert_eq!(loads.get(), 2);
}

#[test]
#[ignore = "porting: system context registry not implemented"]
fn rejects_duplicate_entry_keys() {
    let mut registry = SystemContextRegistry::new();
    registry
        .register(entry("test/duplicate", "first"))
        .expect(NOTE);
    assert!(registry
        .register(entry("test/duplicate", "second"))
        .is_err());
}

#[test]
#[ignore = "porting: system context registry not implemented"]
fn rejects_duplicate_source_keys_from_separate_entries() {
    let mut registry = SystemContextRegistry::new();
    registry
        .register(RegistryEntry {
            key: Key::make("test/first").expect("valid key"),
            load: Box::new(|| Ok(Loaded::Value(json!("first")))),
        })
        .expect(NOTE);
    registry
        .register(RegistryEntry {
            key: Key::make("test/second").expect("valid key"),
            load: Box::new(|| Ok(Loaded::Value(json!("second")))),
        })
        .expect(NOTE);

    // Both entries resolve to the same source key, so loading must fail.
    assert!(registry.load().is_err());
}
