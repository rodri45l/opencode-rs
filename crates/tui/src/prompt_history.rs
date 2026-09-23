//! Prompt history JSONL parsing.
//!
//! Port of packages/tui/src/prompt/history.tsx `parsePromptHistory` and
//! `isDuplicateEntry` behaviour (upstream 18ef3cc).

use serde::{Deserialize, Serialize};

/// The maximum retained history entries.
pub const MAX_HISTORY_ENTRIES: usize = 50;

/// A structured prompt part recorded in history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptPartInfo {
    File {
        mime: String,
        filename: String,
        url: String,
    },
    Agent {
        name: String,
    },
    Text {
        text: String,
    },
}

/// A recorded prompt history entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptInfo {
    pub input: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default)]
    pub parts: Vec<PromptPartInfo>,
}

/// Parse JSONL history, recovering around corruption and keeping the newest.
pub fn parse_prompt_history(text: &str) -> Vec<PromptInfo> {
    let mut entries: Vec<PromptInfo> = text
        .split('\n')
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str::<PromptInfo>(line).ok())
        .collect();
    if entries.len() > MAX_HISTORY_ENTRIES {
        entries.drain(..entries.len() - MAX_HISTORY_ENTRIES);
    }
    entries
}

/// Whether two consecutive history entries are identical.
pub fn is_duplicate_entry(previous: Option<&PromptInfo>, next: &PromptInfo) -> bool {
    previous.is_some_and(|previous| previous == next)
}
