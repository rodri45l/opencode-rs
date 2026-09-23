//! Directory picker domain helpers.
//!
//! Port of `packages/app/src/components/directory-picker-domain.ts` (upstream
//! 18ef3cc). Network-backed search is dropped; the pure path/selection logic is
//! re-derived.

use std::collections::BTreeSet;

/// Map server directory entries into Pierre tree paths.
pub fn tree_entries(prefix: &str, nodes: &[(&str, bool)]) -> Vec<String> {
    let prefix = prefix.trim_matches('/');
    nodes
        .iter()
        .map(|(name, is_directory)| {
            let path = if prefix.is_empty() {
                (*name).to_string()
            } else {
                format!("{prefix}/{name}")
            };
            if *is_directory {
                format!("{path}/")
            } else {
                path
            }
        })
        .collect()
}

/// Map Pierre paths back to an absolute path under the selected root.
pub fn absolute_tree_path(root: &str, path: &str) -> String {
    let base = trim_picker_path(root);
    let relative = path.replace('\\', "/").trim_matches('/').to_string();
    if relative.is_empty() {
        return if base.is_empty() {
            "/".to_string()
        } else {
            base
        };
    }
    if base.is_empty() || base == "/" {
        return format!("/{relative}");
    }
    if base.ends_with('/') {
        format!("{base}{relative}")
    } else {
        format!("{base}/{relative}")
    }
}

/// Filter and map tree entries for the picker mode.
pub fn picker_tree_entries(prefix: &str, nodes: &[(&str, bool)], mode: &str) -> Vec<String> {
    let filtered: Vec<(&str, bool)> = if mode == "directory" {
        nodes.iter().copied().filter(|(_, dir)| *dir).collect()
    } else {
        nodes.to_vec()
    };
    tree_entries(prefix, &filtered)
}

/// Filter search entries for the picker mode.
pub fn picker_search_entries(nodes: &[&str], mode: &str) -> Vec<String> {
    nodes
        .iter()
        .filter(|name| mode != "directory" || !name.contains('.'))
        .map(|name| (*name).to_string())
        .collect()
}

/// Whether a tree mutation belongs to the active navigation.
pub fn active_tree_navigation(token: i64, active: i64) -> bool {
    token == active
}

/// Whether `path` is contained within `root`.
pub fn tree_path_within(root: &str, path: &str) -> bool {
    picker_relative_path(root, path).is_some()
}

/// Display a path using the selected server's format.
pub fn display_picker_path(selected: &str, _path: &str, home: &str) -> String {
    let value = trim_picker_path(selected);
    if is_drive_path(&trim_picker_path(home)) || is_drive_path(&value) {
        return value.replace('/', "\\");
    }
    picker_tilde(&value, home).unwrap_or(value)
}

/// The share/drive/posix root of a path.
pub fn picker_root(input: &str) -> String {
    let value = normalize_picker_drive(input);
    if let Some(stripped) = value.strip_prefix("//") {
        let mut parts = stripped.split('/');
        let server = parts.next().unwrap_or("");
        let share = parts.next().unwrap_or("");
        if !server.is_empty() && !share.is_empty() {
            return format!("//{server}/{share}");
        }
        return "//".to_string();
    }
    if value.starts_with('/') {
        return "/".to_string();
    }
    if is_drive_path(&value) {
        return value[..3].to_string();
    }
    String::new()
}

/// The parent of a path.
pub fn picker_parent(input: &str) -> String {
    let value = trim_picker_path(input);
    let root = picker_root(&value);
    if value == root {
        return value;
    }
    if value == "/" || value == "//" || is_drive_root(&value) {
        return value;
    }
    match value.rfind('/') {
        Some(index) if index < root.len() => root,
        Some(0) => "/".to_string(),
        Some(2) if value.as_bytes()[1] == b':' => value[..3].to_string(),
        Some(index) => value[..index].to_string(),
        None => root,
    }
}

/// Resolve relative input against the current picker root.
pub fn picker_absolute_input(input: &str, home: &str, root: &str) -> String {
    let value = expand_tilde(
        &normalize_picker_drive(input),
        &normalize_picker_drive(home),
    );
    let absolute = if !picker_root(&value).is_empty() {
        value
    } else {
        join_picker_path(root, &value)
    };
    canonical_picker_path(&absolute)
}

/// Return autocomplete results only when they belong to the current query.
pub fn current_picker_suggestions(query: &str, items: &[&str], source_query: &str) -> Vec<String> {
    if query != source_query {
        return Vec::new();
    }
    items.iter().map(|item| (*item).to_string()).collect()
}

/// Scope a file autocomplete query to the current browser root.
pub fn picker_file_search_query(root: &str, path: &str, home: &str) -> String {
    let value = expand_tilde(&path.replace('\\', "/"), home);
    let value = value.trim_end_matches('/').to_string();
    let base = root.replace('\\', "/").trim_end_matches('/').to_string();
    if value == base {
        return String::new();
    }
    if let Some(rest) = value.strip_prefix(&format!("{base}/")) {
        return rest.to_string();
    }
    value
}

/// The next directory level to preload.
pub fn preload_tree_directories(prefix: &str, nodes: &[(&str, bool)]) -> Vec<String> {
    let filtered: Vec<(&str, bool)> = nodes.iter().copied().filter(|(_, dir)| *dir).collect();
    tree_entries(prefix, &filtered)
}

/// Advance preloading once per expanded directory.
pub fn advance_tree_preload(advanced: &mut BTreeSet<String>, path: &str) -> bool {
    advanced.insert(path.to_string())
}

/// Clamp a bridged tree wheel scroll.
pub fn next_tree_scroll_top(
    current: i64,
    delta: i64,
    scroll_height: i64,
    client_height: i64,
) -> i64 {
    let max = (scroll_height - client_height).max(0);
    (current + delta).max(0).min(max)
}

/// Wrap autocomplete keyboard navigation.
pub fn next_suggestion_index(current: i64, delta: i64, length: usize) -> i64 {
    if length == 0 {
        return -1;
    }
    let length = length as i64;
    (current + delta + length) % length
}

/// Return the absolute directory or relative file path for a selection.
pub fn selected_tree_path(root: &str, path: &str, mode: &str) -> Option<String> {
    let directory = path.ends_with('/');
    if mode == "file" {
        if directory {
            return None;
        }
        return Some(path.to_string());
    }
    if directory {
        Some(native_picker_path(&absolute_tree_path(root, path)))
    } else {
        None
    }
}

/// The native (backslash) form of a Windows/UNC path.
pub fn native_picker_path(path: &str) -> String {
    let value = trim_picker_path(path);
    if is_drive_path(&value) || value.starts_with("//") {
        value.replace('/', "\\")
    } else {
        value
    }
}

/// Normalize picker path separators and duplicate slashes.
pub fn normalize_picker_path(input: &str) -> String {
    let value = input.replace('\\', "/");
    if value.starts_with("//") && !value.starts_with("///") {
        return format!("//{}", collapse_slashes(&value[2..]));
    }
    collapse_slashes(&value)
}

/// Normalize a bare drive (`C:`) to `C:/`.
pub fn normalize_picker_drive(input: &str) -> String {
    let value = normalize_picker_path(input);
    if value.len() == 2 && value.as_bytes()[1] == b':' && value.as_bytes()[0].is_ascii_alphabetic()
    {
        return format!("{value}/");
    }
    value
}

/// Trim a picker path, preserving roots.
pub fn trim_picker_path(input: &str) -> String {
    let value = normalize_picker_drive(input);
    if value == "/" || value == "//" || is_drive_root(&value) {
        return value;
    }
    value.trim_end_matches('/').to_string()
}

/// Join a base path and a relative path.
pub fn join_picker_path(base: &str, relative: &str) -> String {
    let root = trim_picker_path(base);
    let path = trim_picker_path(relative)
        .trim_start_matches('/')
        .to_string();
    if root.is_empty() {
        return path;
    }
    if path.is_empty() {
        return root;
    }
    if root.ends_with('/') {
        format!("{root}{path}")
    } else {
        format!("{root}/{path}")
    }
}

/// Canonicalize a picker path (resolving `.` and `..`).
pub fn canonical_picker_path(path: &str) -> String {
    let value = normalize_picker_drive(path);
    let root = picker_root(&value);
    let mut resolved: Vec<&str> = Vec::new();
    for part in value[root.len()..].split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            resolved.pop();
            continue;
        }
        resolved.push(part);
    }
    join_picker_path(&root, &resolved.join("/"))
}

/// The path of `path` relative to `base`, if contained.
pub fn picker_relative_path(base: &str, path: &str) -> Option<String> {
    if base.is_empty() {
        return None;
    }
    let root_path = canonical_picker_path(base);
    let target_path = canonical_picker_path(path);
    let insensitive = is_drive_path(&root_path) || root_path.starts_with("//");
    let root = if insensitive {
        root_path.to_lowercase()
    } else {
        root_path.clone()
    };
    let target = if insensitive {
        target_path.to_lowercase()
    } else {
        target_path.clone()
    };
    if target == root {
        return Some(String::new());
    }
    let prefix = if root.ends_with('/') {
        root.clone()
    } else {
        format!("{root}/")
    };
    if !target.starts_with(&prefix) {
        return None;
    }
    Some(target_path[prefix.len()..].to_string())
}

fn collapse_slashes(value: &str) -> String {
    let mut out = String::new();
    let mut previous_slash = false;
    for character in value.chars() {
        if character == '/' {
            if previous_slash {
                continue;
            }
            previous_slash = true;
        } else {
            previous_slash = false;
        }
        out.push(character);
    }
    out
}

fn expand_tilde(value: &str, home: &str) -> String {
    if value == "~" {
        return home.to_string();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    value.to_string()
}

fn picker_tilde(absolute: &str, home: &str) -> Option<String> {
    let path = trim_picker_path(absolute);
    if home.is_empty() {
        return None;
    }
    let root = trim_picker_path(home);
    if is_drive_path(&root) {
        return None;
    }
    if path == root {
        return Some("~".to_string());
    }
    if let Some(rest) = path.strip_prefix(&format!("{root}/")) {
        return Some(format!("~/{rest}"));
    }
    None
}

fn is_drive_path(value: &str) -> bool {
    value.len() >= 3
        && value.as_bytes()[0].is_ascii_alphabetic()
        && value.as_bytes()[1] == b':'
        && value.as_bytes()[2] == b'/'
}

fn is_drive_root(value: &str) -> bool {
    value.len() == 3
        && value.as_bytes()[0].is_ascii_alphabetic()
        && value.as_bytes()[1] == b':'
        && value.as_bytes()[2] == b'/'
}
