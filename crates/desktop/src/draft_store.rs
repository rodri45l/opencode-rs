//! Buffered draft store.
//!
//! Port of `packages/desktop/src/main/draft-store.ts` (upstream 18ef3cc). The
//! reference is SQLite-backed; this re-derivation keeps the same observable
//! contract (latest buffered draft wins on flush, blobs round-trip by id) with
//! an in-memory backing store.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

/// A buffered desktop draft store.
pub struct DesktopDraftStore {
    documents: RefCell<BTreeMap<String, String>>,
    pending: RefCell<BTreeMap<String, Option<String>>>,
    blobs: RefCell<BTreeMap<String, Vec<u8>>>,
    next_blob: Cell<u64>,
}

impl DesktopDraftStore {
    /// Read a value, preferring the latest buffered draft.
    pub fn get(&self, key: &str) -> String {
        if let Some(value) = self.pending.borrow().get(key) {
            return value.clone().unwrap_or_default();
        }
        self.documents
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    /// Buffer a draft value.
    pub fn set(&self, key: &str, value: &str) {
        self.pending
            .borrow_mut()
            .insert(key.to_string(), Some(value.to_string()));
    }

    /// Persist every buffered draft.
    pub fn flush(&self) {
        let writes: Vec<(String, Option<String>)> = std::mem::take(&mut *self.pending.borrow_mut())
            .into_iter()
            .collect();
        let mut documents = self.documents.borrow_mut();
        for (key, value) in writes {
            match value {
                Some(value) => {
                    documents.insert(key, value);
                }
                None => {
                    documents.remove(&key);
                }
            }
        }
    }

    /// Store a blob and return its id.
    pub fn put_blob(&self, bytes: &[u8]) -> String {
        let id = format!("blob-{}", self.next_blob.get());
        self.next_blob.set(self.next_blob.get() + 1);
        self.blobs.borrow_mut().insert(id.clone(), bytes.to_vec());
        id
    }

    /// Read a blob by id.
    pub fn get_blob(&self, id: &str) -> Vec<u8> {
        self.blobs.borrow().get(id).cloned().unwrap_or_default()
    }

    /// Flush and close the store.
    pub fn close(&self) {
        self.flush();
    }
}

/// Create a draft store. `filename` is accepted for parity but not used by the
/// in-memory backing store.
pub fn create_desktop_draft_store(_filename: &str) -> DesktopDraftStore {
    DesktopDraftStore {
        documents: RefCell::new(BTreeMap::new()),
        pending: RefCell::new(BTreeMap::new()),
        blobs: RefCell::new(BTreeMap::new()),
        next_blob: Cell::new(0),
    }
}
