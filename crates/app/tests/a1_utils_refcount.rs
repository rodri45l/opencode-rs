//! Port of packages/app/src/utils/refcount.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Default)]
struct RefCountMap {
    counts: HashMap<String, usize>,
    removed: Vec<String>,
}

impl RefCountMap {
    // Local stub (fast wave): real module lands later.
    fn acquire(&mut self, _key: &str) {}
    fn release(&mut self, _key: &str) {}
    fn removed(&self) -> &[String] {
        &self.removed
    }
}

#[test]
#[ignore = "porting: utils/refcount not implemented"]
fn removes_an_item_after_its_last_owner_is_disposed() {
    let mut map = RefCountMap::default();
    map.acquire("/project");
    map.acquire("/project");
    map.release("/project");
    assert_eq!(map.removed(), &[] as &[String]);
    map.release("/project");
    assert_eq!(map.removed(), &["/project".to_string()]);
}

#[test]
#[ignore = "porting: utils/refcount not implemented"]
fn keeps_equivalent_path_consumers_until_the_last_owner_is_disposed() {
    let mut map = RefCountMap::default();
    map.acquire("C:\\repo");
    map.acquire("C:/repo/");
    map.release("C:\\repo");
    assert_eq!(map.removed(), &[] as &[String]);
    map.release("C:/repo/");
    assert_eq!(map.removed(), &["C:/repo".to_string()]);
}
