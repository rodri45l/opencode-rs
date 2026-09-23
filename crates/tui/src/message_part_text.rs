//! Message part text reader.
//!
//! Derived from `packages/session-ui/src/components/message-part-text.ts`
//! (upstream 18ef3cc): `readPartText` prefers a trimmed accumulator hit,
//! otherwise trims the part text, returning an empty string for missing or
//! whitespace-only text.

use std::collections::BTreeMap;
use std::fmt;

/// Error raised by the part-text reader.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the part-text reader.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui message part text";

/// One text part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartText {
    pub id: String,
    pub text: Option<String>,
}

/// Read the text for a part, preferring a non-empty accumulator hit.
pub fn read_part_text(
    accum: Option<&BTreeMap<String, String>>,
    part: &PartText,
) -> PortResult<String> {
    if let Some(accum) = accum {
        if let Some(value) = accum.get(&part.id) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Ok(part.text.as_deref().unwrap_or("").trim().to_string())
}
