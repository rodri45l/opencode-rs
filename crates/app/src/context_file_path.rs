//! File path helpers.
//!
//! Port of `packages/app/src/context/file/path.ts` (upstream 18ef3cc).

/// Strip a leading `file://` protocol.
pub fn strip_file_protocol(input: &str) -> String {
    match input.strip_prefix("file://") {
        Some(rest) => rest.to_string(),
        None => input.to_string(),
    }
}

/// Strip a URL query and/or fragment.
pub fn strip_query_and_hash(input: &str) -> String {
    let hash = input.find('#');
    let query = input.find('?');
    let end = match (hash, query) {
        (Some(h), Some(q)) => Some(h.min(q)),
        (Some(h), None) => Some(h),
        (None, Some(q)) => Some(q),
        (None, None) => None,
    };
    match end {
        Some(index) => input[..index].to_string(),
        None => input.to_string(),
    }
}

/// Decode git's quoted/octal-escaped path strings.
pub fn unquote_git_path(input: &str) -> String {
    if !input.starts_with('"') || !input.ends_with('"') {
        return input.to_string();
    }
    let body = &input[1..input.len() - 1];
    let bytes = body.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte != b'\\' {
            out.push(byte);
            index += 1;
            continue;
        }
        let Some(&next) = bytes.get(index + 1) else {
            out.push(b'\\');
            index += 1;
            continue;
        };
        if (b'0'..=b'7').contains(&next) {
            let mut end = index + 1;
            while end < bytes.len() && end < index + 4 && (b'0'..=b'7').contains(&bytes[end]) {
                end += 1;
            }
            let chunk = &body[index + 1..end];
            let value = u8::from_str_radix(chunk, 8).unwrap_or(next);
            out.push(value);
            index = end;
            continue;
        }
        let escaped = match next {
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'b' => 0x08,
            b'f' => 0x0c,
            b'v' => 0x0b,
            b'\\' | b'"' => next,
            other => other,
        };
        out.push(escaped);
        index += 2;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-decode a path (best effort; malformed input is returned unchanged).
pub fn decode_file_path(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &input[index + 1..index + 3];
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// Encode a filesystem path into a `file://` URL path.
pub fn encode_file_path(filepath: &str) -> String {
    let mut normalized = filepath.replace('\\', "/");
    if is_drive_prefix(&normalized) {
        normalized = format!("/{normalized}");
    }
    normalized
        .split('/')
        .enumerate()
        .map(|(index, segment)| {
            if index == 1 && is_drive_segment(segment) {
                segment.to_string()
            } else {
                encode_uri_component(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn is_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_drive_segment(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn encode_uri_component(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            )
        {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Path helpers scoped to a workspace root.
pub struct PathHelpers {
    root: String,
}

impl PathHelpers {
    /// Normalize an input path against the workspace root.
    pub fn normalize(&self, input: &str) -> String {
        let root = &self.root;
        let mut path = unquote_git_path(&decode_file_path(&strip_query_and_hash(
            &strip_file_protocol(input),
        )));

        let windows = is_windows_root(root);
        let canon_root = if windows {
            root.replace('\\', "/").to_lowercase()
        } else {
            root.replace('\\', "/")
        };
        let canon_path = if windows {
            path.replace('\\', "/").to_lowercase()
        } else {
            path.replace('\\', "/")
        };
        if canon_path.starts_with(&canon_root)
            && (canon_root.ends_with('/')
                || canon_path == canon_root
                || canon_path.as_bytes().get(canon_root.len()) == Some(&b'/'))
        {
            path = path[root.len()..].to_string();
        }

        if path.starts_with("./") || path.starts_with(".\\") {
            path = path[2..].to_string();
        }
        if path.starts_with('/') || path.starts_with('\\') {
            path = path[1..].to_string();
        }
        path
    }

    /// Normalize a directory path, collapsing trailing separators.
    pub fn normalize_dir(&self, input: &str) -> String {
        let path = self.normalize(input);
        let windows = is_windows_root(&self.root);
        let path = if windows {
            path.replace('\\', "/")
        } else {
            path
        };
        path.trim_end_matches('/').to_string()
    }

    /// Build the tab identifier for a path.
    pub fn tab(&self, input: &str) -> String {
        let path = self.normalize(input);
        format!("file://{}", encode_file_path(&path))
    }

    /// Recover the path from a tab identifier.
    pub fn path_from_tab(&self, tab_value: &str) -> Option<String> {
        if !tab_value.starts_with("file://") {
            return None;
        }
        Some(self.normalize(tab_value))
    }
}

fn is_windows_root(root: &str) -> bool {
    is_drive_prefix(root) || root.starts_with("\\\\")
}

/// Create path helpers scoped to `root`.
pub fn create_path_helpers(root: &str) -> PathHelpers {
    PathHelpers {
        root: root.to_string(),
    }
}
