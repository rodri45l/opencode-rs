//! Port of packages/desktop/src/main/draft-store.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/draft-store.ts: the latest
//! buffered draft is what a flush persists, and blobs round-trip by id.

#[allow(dead_code)]
mod draft_store {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop draft store not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    pub struct DesktopDraftStore;

    impl DesktopDraftStore {
        pub fn set(&self, _key: &str, _value: &str) {}

        pub fn get(&self, _key: &str) -> PortResult<String> {
            stub()
        }

        pub fn flush(&self) {}

        pub fn put_blob(&self, _bytes: &[u8]) -> PortResult<String> {
            stub()
        }

        pub fn get_blob(&self, _id: &str) -> PortResult<Vec<u8>> {
            stub()
        }

        pub fn close(&self) {}
    }

    pub fn create_desktop_draft_store(_path: &str) -> PortResult<DesktopDraftStore> {
        stub()
    }
}

use draft_store::{create_desktop_draft_store, NOTE};

#[test]
#[ignore = "porting: desktop draft store not implemented"]
fn flushes_the_latest_buffered_draft_and_stores_blobs() {
    let store = create_desktop_draft_store(":memory:").expect(NOTE);
    store.set("prompt", "first");
    store.set("prompt", "latest");
    assert_eq!(store.get("prompt").expect(NOTE), "latest");
    store.flush();
    assert_eq!(store.get("prompt").expect(NOTE), "latest");

    let id = store.put_blob(b"image").expect(NOTE);
    assert_eq!(store.get_blob(&id).expect(NOTE), b"image".to_vec());
    store.close();
}
