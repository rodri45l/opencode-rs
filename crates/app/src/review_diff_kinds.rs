//! Review diff classification (port of packages/app/src/pages/session/v2/review-diff-kinds.ts).

use std::collections::BTreeMap;

use crate::file_tree_v2_model::normalize_file_tree_v2_path;

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewDiff {
    pub file: String,
    pub additions: i64,
    pub deletions: i64,
    pub status: Option<String>,
    pub patch: Option<String>,
}

pub fn normalize_path(value: &str) -> String {
    normalize_file_tree_v2_path(value)
}

fn merge(current: Option<&String>, kind: &str) -> String {
    match current {
        None => kind.to_string(),
        Some(existing) if existing == kind => existing.clone(),
        Some(_) => "mix".to_string(),
    }
}

pub fn review_diff_kinds(diffs: &[ReviewDiff]) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for diff in diffs {
        let file = normalize_path(&diff.file);
        let kind = match diff.status.as_deref() {
            Some("added") => "add",
            Some("deleted") => "del",
            _ => "mix",
        };
        out.insert(file.clone(), kind.to_string());

        let parts: Vec<&str> = file.split('/').collect();
        for index in 1..parts.len() {
            let dir = parts[..index].join("/");
            if dir.is_empty() {
                continue;
            }
            let merged = merge(out.get(&dir), kind);
            out.insert(dir, merged);
        }
    }
    out
}

pub fn filter_review_files(files: &[&str], query: &str) -> Vec<String> {
    let value = query.trim().to_lowercase();
    if value.is_empty() {
        return files.iter().map(|file| (*file).to_string()).collect();
    }
    files
        .iter()
        .filter(|file| file.to_lowercase().contains(&value))
        .map(|file| (*file).to_string())
        .collect()
}

pub fn review_diff_needs_load(diff: &ReviewDiff) -> bool {
    if diff.additions == 0 && diff.deletions == 0 {
        return false;
    }
    match &diff.patch {
        None => true,
        Some(patch) => !patch.lines().any(|line| line.starts_with("@@ ")),
    }
}

fn review_root_directory(root: &str) -> String {
    if root == "/" || is_drive_root(root) {
        return root.to_string();
    }
    root.trim_end_matches(['/', '\\']).to_string()
}

fn is_drive_root(root: &str) -> bool {
    let bytes = root.as_bytes();
    if bytes.len() == 2 {
        return bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    }
    if bytes.len() == 3 {
        return bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'/' || bytes[2] == b'\\');
    }
    false
}

pub fn review_diff_directory(root: &str, file: &str) -> String {
    let path = normalize_path(file);
    let index = path.rfind('/');
    let separator = if root.contains('\\') { '\\' } else { '/' };
    let base = review_root_directory(root);
    let index = match index {
        Some(index) => index,
        None => return base,
    };
    let joined = if base.ends_with(separator) {
        base
    } else {
        format!("{base}{separator}")
    };
    let dir = path[..index].replace('/', &separator.to_string());
    format!("{joined}{dir}")
}
