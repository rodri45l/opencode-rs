//! Port of packages/app/src/utils/scoped-cache.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    key: String,
    count: i64,
}

struct ScopedCache {
    max_entries: Option<usize>,
    ttl_ms: Option<i64>,
    clock: i64,
    disposed: Vec<String>,
}

impl ScopedCache {
    fn new(max_entries: Option<usize>, ttl_ms: Option<i64>) -> Self {
        ScopedCache {
            max_entries,
            ttl_ms,
            clock: 0,
            disposed: Vec::new(),
        }
    }

    // Local stubs (fast wave): real module lands later.
    fn get(&mut self, _key: &str) -> Entry {
        Entry {
            key: String::new(),
            count: 0,
        }
    }
    fn peek(&self, _key: &str) -> Option<Entry> {
        None
    }
    fn delete(&mut self, _key: &str) -> Option<Entry> {
        None
    }
    fn clear(&mut self) {}
    fn set_clock(&mut self, clock: i64) {
        self.clock = clock;
    }
    fn disposed(&self) -> &[String] {
        &self.disposed
    }
}

#[test]
#[ignore = "porting: utils/scoped-cache not implemented"]
fn evicts_least_recently_used_entry_when_max_is_reached() {
    let mut cache = ScopedCache::new(Some(2), None);
    let a = cache.get("a");
    let b = cache.get("b");
    assert_eq!(a.key, "a");
    assert_eq!(b.key, "b");

    cache.get("a");
    let c = cache.get("c");

    assert_eq!(c.key, "c");
    assert_eq!(cache.peek("a").map(|e| e.key), Some("a".to_string()));
    assert_eq!(cache.peek("b"), None);
    assert_eq!(cache.peek("c").map(|e| e.key), Some("c".to_string()));
    assert_eq!(cache.disposed(), &["b".to_string()]);
}

#[test]
#[ignore = "porting: utils/scoped-cache not implemented"]
fn disposes_entries_on_delete_and_clear() {
    let mut cache = ScopedCache::new(None, None);
    cache.get("a");
    cache.get("b");

    let removed = cache.delete("a");
    assert_eq!(removed.map(|e| e.key), Some("a".to_string()));
    assert_eq!(cache.peek("a"), None);

    cache.clear();
    assert_eq!(cache.peek("b"), None);
    assert_eq!(cache.disposed(), &["a".to_string(), "b".to_string()]);
}

#[test]
#[ignore = "porting: utils/scoped-cache not implemented"]
fn expires_stale_entries_with_ttl_and_recreates_on_get() {
    let mut cache = ScopedCache::new(None, Some(10));
    let first = cache.get("a");
    assert_eq!(first.count, 1);

    cache.set_clock(9);
    assert_eq!(cache.peek("a").map(|e| e.count), Some(1));

    cache.set_clock(11);
    assert_eq!(cache.peek("a"), None);
    assert_eq!(cache.disposed(), &["a:1".to_string()]);

    let second = cache.get("a");
    assert_eq!(second.count, 2);
    assert_eq!(cache.disposed(), &["a:1".to_string()]);
}
