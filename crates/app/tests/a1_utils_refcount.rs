//! Port of packages/app/src/utils/refcount.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::refcount::RefCountMap;

#[test]
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
fn keeps_equivalent_path_consumers_until_the_last_owner_is_disposed() {
    let mut map = RefCountMap::default();
    map.acquire("C:\\repo");
    map.acquire("C:/repo/");
    map.release("C:\\repo");
    assert_eq!(map.removed(), &[] as &[String]);
    map.release("C:/repo/");
    assert_eq!(map.removed(), &["C:/repo".to_string()]);
}
