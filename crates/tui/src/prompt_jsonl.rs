//! Prompt stash and frecency JSONL parsing.
//!
//! Port of packages/tui/src/prompt/stash.tsx `parsePromptStash` and
//! packages/tui/src/prompt/frecency.tsx `parseFrecency` behaviour (upstream
//! 18ef3cc).

use crate::prompt_history::PromptPartInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The maximum retained stash entries.
pub const MAX_STASH_ENTRIES: usize = 50;
/// The maximum retained frecency entries.
pub const MAX_FRECENCY_ENTRIES: usize = 1000;

/// A stashed prompt entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashEntry {
    pub input: String,
    #[serde(default)]
    pub parts: Vec<PromptPartInfo>,
    pub timestamp: i64,
}

/// A frecency entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrecencyEntry {
    pub path: String,
    pub frequency: i64,
    #[serde(rename = "lastOpen")]
    pub last_open: i64,
}

/// Parse JSONL stash entries, skipping corruption and keeping the newest.
pub fn parse_prompt_stash(text: &str) -> Vec<StashEntry> {
    let mut entries: Vec<StashEntry> = text
        .split('\n')
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str::<StashEntry>(line).ok())
        .collect();
    if entries.len() > MAX_STASH_ENTRIES {
        entries.drain(..entries.len() - MAX_STASH_ENTRIES);
    }
    entries
}

/// Parse JSONL frecency entries, keeping the latest state per path.
pub fn parse_frecency(text: &str) -> Vec<FrecencyEntry> {
    let mut latest: HashMap<String, FrecencyEntry> = HashMap::new();
    for line in text.split('\n').filter(|line| !line.is_empty()) {
        if let Ok(entry) = serde_json::from_str::<FrecencyEntry>(line) {
            latest.insert(entry.path.clone(), entry);
        }
    }
    let mut entries: Vec<FrecencyEntry> = latest.into_values().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.last_open));
    entries.truncate(MAX_FRECENCY_ENTRIES);
    entries
}
