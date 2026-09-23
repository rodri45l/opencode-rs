//! Port of packages/app/src/context/layout.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::BTreeMap;

use opencode_app::layout_helpers::{
    create_session_key_reader, ensure_session_key, prune_session_keys, PruneInput,
};

#[test]
fn couples_touch_and_scroll_seed_in_order() {
    let calls = RefCell::new(Vec::new());
    let result = ensure_session_key(
        "dir/a",
        |key| calls.borrow_mut().push(format!("touch:{key}")),
        |key| calls.borrow_mut().push(format!("seed:{key}")),
    );
    assert_eq!(result, "dir/a");
    assert_eq!(
        calls.into_inner(),
        vec!["touch:dir/a".to_string(), "seed:dir/a".to_string()]
    );
}

#[test]
fn reads_dynamic_accessor_keys_lazily() {
    let mut seen: Vec<String> = Vec::new();
    let first = create_session_key_reader("dir/one");
    seen.push(first.clone());
    let second = create_session_key_reader("dir/two");
    seen.push(second);
    assert_eq!(seen, vec!["dir/one".to_string(), "dir/two".to_string()]);
}

#[test]
fn keeps_active_key_and_drops_lowest_used_keys() {
    let mut used = BTreeMap::new();
    used.insert("k1".to_string(), 1);
    used.insert("k2".to_string(), 2);
    used.insert("k3".to_string(), 3);
    used.insert("k4".to_string(), 4);
    let drop = prune_session_keys(PruneInput {
        keep: Some("k4".into()),
        max: 3,
        used,
        view: vec!["k1".into(), "k2".into(), "k4".into()],
        tabs: vec!["k1".into(), "k3".into(), "k4".into()],
    });
    assert_eq!(drop, vec!["k1".to_string()]);
    assert!(!drop.contains(&"k4".to_string()));
}

#[test]
fn does_not_prune_without_keep_key() {
    let mut used = BTreeMap::new();
    used.insert("k1".to_string(), 1);
    used.insert("k2".to_string(), 2);
    let drop = prune_session_keys(PruneInput {
        keep: None,
        max: 1,
        used,
        view: vec!["k1".into()],
        tabs: vec!["k2".into()],
    });
    assert!(drop.is_empty());
}
