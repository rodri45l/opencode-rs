//! Attachment picker budget and authorization.
//!
//! Port of `packages/desktop/src/main/attachment-picker.ts` (upstream 18ef3cc).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use crate::error::AttachmentError;

/// The media ingest limit (20 MB).
pub const MAX_ATTACHMENT_BYTES: u64 = 20 * 1024 * 1024;

/// Reject a selection whose total size exceeds the ingest limit.
pub fn assert_attachment_budget(sizes: &[u64]) -> Result<(), AttachmentError> {
    let total: u64 = sizes.iter().sum();
    if total > MAX_ATTACHMENT_BYTES {
        return Err(AttachmentError(format!(
            "Attachment selection exceeds the 20 MB limit ({MAX_ATTACHMENT_BYTES} bytes)"
        )));
    }
    Ok(())
}

/// Read an approved file, rejecting oversized files before allocating.
pub fn read_attachment(path: &Path) -> Result<Vec<u8>, AttachmentError> {
    let metadata = std::fs::metadata(path).map_err(|error| AttachmentError(error.to_string()))?;
    if metadata.len() > MAX_ATTACHMENT_BYTES {
        return Err(AttachmentError("20 MB limit".to_string()));
    }
    std::fs::read(path).map_err(|error| AttachmentError(error.to_string()))
}

struct Selection {
    sender: u64,
    paths: Vec<String>,
    remaining: u64,
}

type ReadFn = Box<dyn Fn(&str, u64) -> u64>;

/// Per-renderer, per-token picked-file authorizations.
pub struct PickerAuthorizations {
    selections: RefCell<BTreeMap<u64, Selection>>,
    next_token: u64,
    read: ReadFn,
    budget: u64,
}

impl PickerAuthorizations {
    /// Authorize a new selection, returning its token.
    pub fn add(&mut self, renderer: u64, paths: &[&str]) -> u64 {
        let token = self.next_token;
        self.next_token += 1;
        self.selections.borrow_mut().insert(
            token,
            Selection {
                sender: renderer,
                paths: paths.iter().map(|path| (*path).to_string()).collect(),
                remaining: self.budget,
            },
        );
        token
    }

    /// Read an authorized path, charging the actual read against the budget.
    pub fn read(&self, renderer: u64, token: u64, path: &str) -> Result<Vec<u8>, AttachmentError> {
        let mut selections = self.selections.borrow_mut();
        let selection = selections
            .get_mut(&token)
            .ok_or_else(|| AttachmentError("not selected".to_string()))?;
        if selection.sender != renderer {
            return Err(AttachmentError("not selected".to_string()));
        }
        let index = selection
            .paths
            .iter()
            .position(|candidate| candidate == path)
            .ok_or_else(|| AttachmentError("not selected".to_string()))?;
        selection.paths.remove(index);

        let actual = (self.read)(path, selection.remaining);
        if selection.remaining > 0 && actual == 0 {
            return Err(AttachmentError("budget exceeded".to_string()));
        }
        selection.remaining = selection.remaining.saturating_sub(actual);
        if selection.paths.is_empty() {
            selections.remove(&token);
        }
        Ok(path.as_bytes().to_vec())
    }

    /// Release a selection.
    pub fn release(&mut self, renderer: u64, token: u64) {
        let matches = self
            .selections
            .borrow()
            .get(&token)
            .map(|selection| selection.sender == renderer)
            .unwrap_or(false);
        if matches {
            self.selections.borrow_mut().remove(&token);
        }
    }
}

/// Create the picked-file authorization store.
pub fn create_picked_file_authorizations(
    read: impl Fn(&str, u64) -> u64 + 'static,
    budget: u64,
) -> PickerAuthorizations {
    PickerAuthorizations {
        selections: RefCell::new(BTreeMap::new()),
        next_token: 0,
        read: Box::new(read),
        budget,
    }
}
