//! Port of packages/app/src/context/layout-scroll.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use std::collections::BTreeMap;

use opencode_app::layout_scroll::{Point, ScrollPersistence, Snapshot};

#[test]
fn debounces_persisted_scroll_writes() {
    let mut initial = Snapshot::new();
    let mut inner = BTreeMap::new();
    inner.insert("review".to_string(), Point { x: 0, y: 0 });
    initial.insert("session".to_string(), inner);

    let mut scroll = ScrollPersistence::new(initial, 10);
    for i in 1..=30 {
        scroll.set_scroll("session", "review", Point { x: 0, y: i });
    }

    scroll.advance(9);
    assert!(scroll.writes.is_empty());

    scroll.advance(1);
    assert_eq!(scroll.writes.len(), 1);
    assert_eq!(
        scroll.writes[0]
            .get("session")
            .and_then(|s| s.get("review")),
        Some(&Point { x: 0, y: 30 })
    );

    scroll.set_scroll("session", "review", Point { x: 0, y: 30 });
    scroll.advance(20);
    assert_eq!(scroll.writes.len(), 1);
    scroll.dispose();
}

#[test]
fn reseeds_empty_cache_after_persisted_snapshot_loads() {
    let mut initial = Snapshot::new();
    initial.insert("session".to_string(), BTreeMap::new());
    let mut scroll = ScrollPersistence::new(initial, 0);
    assert_eq!(scroll.scroll("session", "review"), None);

    if let Some(session) = scroll.snapshot.get_mut("session") {
        session.insert("review".to_string(), Point { x: 12, y: 34 });
    }
    assert_eq!(
        scroll.scroll("session", "review"),
        Some(Point { x: 12, y: 34 })
    );
    scroll.dispose();
}
