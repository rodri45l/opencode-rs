//! Message part text extraction.
//!
//! Port of packages/session-ui/src/components/message-part-text.ts behaviour
//! (upstream 18ef3cc).

use std::collections::HashMap;

/// Read a part's text, preferring an accumulated stream value when present.
pub fn read_part_text(
    accum: Option<&HashMap<String, String>>,
    part_id: &str,
    text: Option<&str>,
) -> String {
    if let Some(value) = accum.and_then(|accum| accum.get(part_id)) {
        return value.trim().to_string();
    }
    text.unwrap_or("").trim().to_string()
}
