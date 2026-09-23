//! Desktop store cleanup.
//!
//! Port of `packages/desktop/src/main/store-cleanup.ts` (upstream 18ef3cc):
//! empty scoped stores are deleted, stale drafts expire after 30 days, and
//! scoped stores are capped to the 100 most recent drafts.

const EMPTY_STORE_MAX_BYTES: usize = 128;
const DRAFT_RETENTION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
const DRAFT_KEEP_RECENT: usize = 100;

/// An abstracted store file.
#[derive(Debug, Clone)]
pub struct StoreFile {
    pub name: String,
    pub contents: String,
    pub modified_ms: i64,
}

impl StoreFile {
    pub fn new(name: &str, contents: &str, modified_ms: i64) -> Self {
        Self {
            name: name.to_string(),
            contents: contents.to_string(),
            modified_ms,
        }
    }
}

/// The cleanup outcome.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CleanupResult {
    pub deleted: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreKind {
    Draft,
    Workspace,
}

/// Delete empty scoped stores, stale drafts, and surplus drafts by recency.
pub fn cleanup_store_files(files: &[StoreFile], now_ms: i64) -> CleanupResult {
    let candidates: Vec<(&StoreFile, StoreKind)> = files
        .iter()
        .filter_map(|file| store_kind(&file.name).map(|kind| (file, kind)))
        .collect();

    let mut deleted: Vec<String> = Vec::new();
    for (file, kind) in &candidates {
        let empty = is_empty_store(&file.contents);
        let stale_draft =
            *kind == StoreKind::Draft && now_ms - file.modified_ms > DRAFT_RETENTION_MS;
        if empty || stale_draft {
            deleted.push(file.name.clone());
        }
    }

    let mut non_empty_drafts: Vec<&StoreFile> = candidates
        .iter()
        .filter(|(file, kind)| *kind == StoreKind::Draft && !is_empty_store(&file.contents))
        .map(|(file, _)| *file)
        .collect();
    non_empty_drafts.sort_by_key(|a| std::cmp::Reverse(a.modified_ms));
    for file in non_empty_drafts.into_iter().skip(DRAFT_KEEP_RECENT) {
        deleted.push(file.name.clone());
    }

    CleanupResult { deleted }
}

/// Delete a scoped store file if it is empty.
pub fn delete_store_file_if_empty(files: &[StoreFile], name: &str) -> bool {
    if store_kind(name).is_none() {
        return false;
    }
    match files.iter().find(|file| file.name == name) {
        Some(file) => is_empty_store(&file.contents),
        None => false,
    }
}

fn store_kind(name: &str) -> Option<StoreKind> {
    if scoped_dat(name, "opencode.draft.") {
        return Some(StoreKind::Draft);
    }
    if scoped_dat(name, "opencode.workspace.") {
        return Some(StoreKind::Workspace);
    }
    None
}

fn scoped_dat(name: &str, prefix: &str) -> bool {
    name.starts_with(prefix) && name.ends_with(".dat") && name.len() > prefix.len() + ".dat".len()
}

fn is_empty_store(contents: &str) -> bool {
    if contents.len() > EMPTY_STORE_MAX_BYTES {
        return false;
    }
    if contents.trim().is_empty() {
        return true;
    }
    match serde_json::from_str::<serde_json::Value>(contents) {
        Ok(serde_json::Value::Object(map)) => map.is_empty(),
        _ => false,
    }
}
