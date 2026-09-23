//! Path identity keys (port of packages/app/src/utils/path-key.ts).

fn is_drive(value: &str) -> bool {
    if value.len() != 2 {
        return false;
    }
    let first = value.as_bytes()[0];
    value.as_bytes()[1] == b':' && first.is_ascii_alphabetic()
}

fn trim_trailing_slashes(value: &str) -> &str {
    let trimmed = value.trim_end_matches('/');
    trimmed
}

fn is_windows_path(value: &str) -> bool {
    value.as_bytes().get(1) == Some(&b':') || value.starts_with("\\\\")
}

/// Normalise a path into a stable identity key.
pub fn path_key(path: &str) -> String {
    let value = if is_windows_path(path) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };
    let trimmed = trim_trailing_slashes(&value);
    if trimmed.is_empty() && value.starts_with('/') {
        return "/".to_string();
    }
    if is_drive(trimmed) {
        return format!("{trimmed}/");
    }
    trimmed.to_string()
}
