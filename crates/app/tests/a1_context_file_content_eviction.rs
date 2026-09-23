//! Port of packages/app/src/context/file-content-eviction-accounting.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

const CHUNK: i64 = 8 * 1024 * 1024;

#[derive(Default)]
struct ContentCache {
    entries: BTreeMap<String, i64>,
    order: Vec<String>,
}

impl ContentCache {
    // Local stubs (fast wave): real module lands later.
    fn set(&mut self, _path: &str, _bytes: i64) {}
    fn touch(&mut self, _path: &str) {}
    fn remove(&mut self, _path: &str) {}
    fn reset(&mut self) {}
    fn bytes_total(&self) -> i64 {
        0
    }
    fn entry_count(&self) -> usize {
        0
    }
    fn evict(&mut self, _protected: Option<Vec<String>>) -> Vec<String> {
        Vec::new()
    }
}

#[test]
#[ignore = "porting: context/file-content-eviction-accounting not implemented"]
fn updates_byte_totals_incrementally_for_set_overwrite_remove_and_reset() {
    let mut cache = ContentCache::default();
    cache.set("a", 10);
    cache.set("b", 15);
    assert_eq!(cache.bytes_total(), 25);
    assert_eq!(cache.entry_count(), 2);

    cache.set("a", 5);
    assert_eq!(cache.bytes_total(), 20);
    assert_eq!(cache.entry_count(), 2);

    cache.touch("a");
    assert_eq!(cache.bytes_total(), 20);

    cache.remove("b");
    assert_eq!(cache.bytes_total(), 5);
    assert_eq!(cache.entry_count(), 1);

    cache.reset();
    assert_eq!(cache.bytes_total(), 0);
    assert_eq!(cache.entry_count(), 0);
}

#[test]
#[ignore = "porting: context/file-content-eviction-accounting not implemented"]
fn evicts_by_entry_cap_using_lru_order() {
    let mut cache = ContentCache::default();
    for i in 0..41 {
        cache.set(&format!("f-{i}"), 1);
    }
    let evicted = cache.evict(None);
    assert_eq!(evicted, vec!["f-0".to_string()]);
    assert_eq!(cache.entry_count(), 40);
    assert_eq!(cache.bytes_total(), 40);
}

#[test]
#[ignore = "porting: context/file-content-eviction-accounting not implemented"]
fn evicts_by_byte_cap_while_preserving_protected_entries() {
    let mut cache = ContentCache::default();
    cache.set("a", CHUNK);
    cache.set("b", CHUNK);
    cache.set("c", CHUNK);
    let evicted = cache.evict(Some(vec!["a".to_string()]));
    assert_eq!(evicted, vec!["b".to_string()]);
    assert_eq!(cache.entry_count(), 2);
    assert_eq!(cache.bytes_total(), CHUNK * 2);
}
