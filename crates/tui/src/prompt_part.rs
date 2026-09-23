//! Prompt part helpers.
//!
//! Port of packages/tui/src/prompt/part.ts `stripPromptPartIDs` and
//! `expandTrackedPastedText` (upstream 18ef3cc).

use crate::prompt_display::{display_slice, prompt_offset_width};

/// A persisted file part with its storage ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptFilePart {
    pub id: String,
    pub message_id: String,
    pub session_id: String,
    pub kind: String,
    pub mime: String,
    pub filename: String,
    pub url: String,
}

/// A file part without its storage ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptFileContent {
    pub kind: String,
    pub mime: String,
    pub filename: String,
    pub url: String,
}

/// Strip the persisted ids from a reused part.
pub fn strip_prompt_part_ids(part: PromptFilePart) -> PromptFileContent {
    PromptFileContent {
        kind: part.kind,
        mime: part.mime,
        filename: part.filename,
        url: part.url,
    }
}

/// A tracked pasted-text range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteRange {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Expand tracked pasted-text placeholders by display column.
pub fn expand_tracked_pasted_text(text: &str, ranges: &[PasteRange]) -> String {
    let mut sorted: Vec<&PasteRange> = ranges.iter().collect();
    sorted.sort_by_key(|part| std::cmp::Reverse(part.start));
    sorted.into_iter().fold(text.to_string(), |result, part| {
        let end = prompt_offset_width(&result);
        format!(
            "{}{}{}",
            display_slice(&result, 0, part.start),
            part.text,
            display_slice(&result, part.end, end)
        )
    })
}
