//! Port of packages/desktop/src/main/draft-store.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/draft-store.ts: the latest
//! buffered draft is what a flush persists, and blobs round-trip by id.

use opencode_desktop::draft_store::create_desktop_draft_store;

#[test]
fn flushes_the_latest_buffered_draft_and_stores_blobs() {
    let store = create_desktop_draft_store(":memory:");
    store.set("prompt", "first");
    store.set("prompt", "latest");
    assert_eq!(store.get("prompt"), "latest");
    store.flush();
    assert_eq!(store.get("prompt"), "latest");

    let id = store.put_blob(b"image");
    assert_eq!(store.get_blob(&id), b"image".to_vec());
    store.close();
}
