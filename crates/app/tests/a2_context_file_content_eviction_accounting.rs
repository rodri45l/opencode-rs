//! Port of packages/app/src/context/file-content-eviction-accounting.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

const ENTRY_CAP: usize = 40;
const BYTE_CAP: usize = 32 * 1024 * 1024;

#[derive(Default)]
struct FileContentCache;

impl FileContentCache {
    // Local stubs (fast wave): real module lands later.
    fn set_bytes(&self, _path: &str, _bytes: usize) {}
    fn bytes_total(&self) -> usize {
        0
    }
    fn entry_count(&self) -> usize {
        0
    }
    fn touch(&self, _path: &str) {}
    fn remove(&self, _path: &str) {}
    fn reset(&self) {}
    fn evict_lru(&self, _protected: Option<&[&str]>, _on_evict: &mut dyn FnMut(&str)) {}
}

#[test]
#[ignore = "porting: context/file content cache not implemented"]
fn updates_byte_totals_incrementally_for_set_overwrite_remove_and_reset() {
    let cache = FileContentCache;

    cache.set_bytes("a", 10);
    cache.set_bytes("b", 15);
    assert_eq!(cache.bytes_total(), 25);
    assert_eq!(cache.entry_count(), 2);

    cache.set_bytes("a", 5);
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
#[ignore = "porting: context/file content cache not implemented"]
fn evicts_by_entry_cap_using_lru_order() {
    let cache = FileContentCache;
    for i in 0..41 {
        cache.set_bytes(&format!("f-{i}"), 1);
    }

    let mut evicted: Vec<String> = Vec::new();
    cache.evict_lru(None, &mut |path| evicted.push(path.to_string()));

    assert_eq!(evicted, vec!["f-0".to_string()]);
    assert_eq!(cache.entry_count(), ENTRY_CAP);
    assert_eq!(cache.bytes_total(), ENTRY_CAP);
}

#[test]
#[ignore = "porting: context/file content cache not implemented"]
fn evicts_by_byte_cap_while_preserving_protected_entries() {
    let cache = FileContentCache;
    let chunk = 8 * 1024 * 1024;
    cache.set_bytes("a", chunk);
    cache.set_bytes("b", chunk);
    cache.set_bytes("c", chunk);

    let mut evicted: Vec<String> = Vec::new();
    cache.evict_lru(Some(&["a"]), &mut |path| evicted.push(path.to_string()));

    assert_eq!(evicted, vec!["b".to_string()]);
    assert_eq!(cache.entry_count(), 2);
    assert_eq!(cache.bytes_total(), chunk * 2);
    const { assert!(BYTE_CAP > 0) };
}
