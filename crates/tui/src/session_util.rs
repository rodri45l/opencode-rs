//! Session title helpers.
//!
//! Port of packages/tui/src/util/session.ts `isDefaultTitle` (upstream 18ef3cc).

/// Whether a session title is a generated default title.
pub fn is_default_title(title: &str) -> bool {
    let rest = if let Some(rest) = title.strip_prefix("New session - ") {
        rest
    } else if let Some(rest) = title.strip_prefix("Child session - ") {
        rest
    } else {
        return false;
    };
    is_iso_timestamp(rest)
}

fn is_iso_timestamp(value: &str) -> bool {
    // `YYYY-MM-DDTHH:MM:SS.mmmZ`
    if value.len() != 24 {
        return false;
    }
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        let expected_digit = !matches!(index, 4 | 7 | 10 | 13 | 16 | 19 | 23);
        if expected_digit && !byte.is_ascii_digit() {
            return false;
        }
    }
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'.'
        && bytes[23] == b'Z'
}
