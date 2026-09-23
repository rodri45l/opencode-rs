//! Apply-patch file projection.
//!
//! Port of packages/session-ui/src/components/apply-patch-file.ts behaviour
//! (upstream 18ef3cc).

use crate::session_diff::{normalize, DiffSource, ViewDiff};

/// A raw apply-patch file payload.
#[derive(Debug, Clone, Default)]
pub struct ApplyPatchRaw {
    pub file_path: Option<String>,
    pub relative_path: Option<String>,
    pub kind: Option<String>,
    pub patch: Option<String>,
    pub diff: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub additions: Option<i64>,
    pub deletions: Option<i64>,
    pub move_path: Option<String>,
}

/// A resolved apply-patch file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchFile {
    pub file_path: String,
    pub relative_path: String,
    pub kind: String,
    pub additions: i64,
    pub deletions: i64,
    pub move_path: Option<String>,
    pub view: ViewDiff,
}

/// Project a single raw apply-patch file, or `None` when it is not usable.
pub fn patch_file(raw: &ApplyPatchRaw) -> Option<ApplyPatchFile> {
    let kind = normalize_kind(raw.kind.as_deref())?;
    let file_path = raw.file_path.clone()?;
    let relative_path = raw
        .relative_path
        .clone()
        .unwrap_or_else(|| file_path.clone());
    let patch = raw.patch.clone().or_else(|| raw.diff.clone());
    let before = raw.before.clone();
    let after = raw.after.clone();
    if patch.is_none() && before.is_none() && after.is_none() {
        return None;
    }
    let additions = raw.additions.unwrap_or(0);
    let deletions = raw.deletions.unwrap_or(0);
    let view = normalize(&DiffSource {
        file: relative_path.clone(),
        patch,
        before,
        after,
        additions,
        deletions,
        status: Some(status_for(&kind).to_string()),
    });
    Some(ApplyPatchFile {
        file_path,
        relative_path,
        kind,
        additions,
        deletions,
        move_path: raw.move_path.clone(),
        view,
    })
}

/// Project a list of raw apply-patch files, dropping unusable entries.
pub fn patch_files(raw: &[ApplyPatchRaw]) -> Vec<ApplyPatchFile> {
    raw.iter().filter_map(patch_file).collect()
}

fn normalize_kind(value: Option<&str>) -> Option<String> {
    match value {
        Some("add") | Some("update") | Some("delete") | Some("move") => value.map(str::to_string),
        _ => None,
    }
}

fn status_for(kind: &str) -> &'static str {
    match kind {
        "add" => "added",
        "delete" => "deleted",
        _ => "modified",
    }
}
