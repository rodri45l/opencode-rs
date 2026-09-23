//! Port of packages/tui/test/prompt/jsonl.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/prompt/frecency.tsx and
//! packages/tui/src/prompt/stash.tsx; see docs/TEST-PORT.md.

use opencode_tui::prompt_jsonl::{
    parse_frecency, parse_prompt_stash, FrecencyEntry, MAX_FRECENCY_ENTRIES, MAX_STASH_ENTRIES,
};
use serde_json::json;

#[test]
fn stash_jsonl_skips_corruption_and_retains_newest_entries() {
    let mut entries: Vec<String> = (0..MAX_STASH_ENTRIES + 2)
        .map(|index| {
            json!({ "input": index.to_string(), "parts": [], "timestamp": index }).to_string()
        })
        .collect();
    entries.insert(2, "broken".to_string());
    let result = parse_prompt_stash(&entries.join("\n"));
    assert_eq!(result.len(), MAX_STASH_ENTRIES);
    assert_eq!(result[0].input, "2");
}

#[test]
fn frecency_jsonl_skips_corruption_keeps_latest_path_state_and_limits_entries() {
    let mut entries: Vec<String> = (0..=MAX_FRECENCY_ENTRIES)
        .map(|index| {
            json!({ "path": index.to_string(), "frequency": 1, "lastOpen": index }).to_string()
        })
        .collect();
    entries.push("broken".to_string());
    entries.push(json!({ "path": "1000", "frequency": 2, "lastOpen": 2000 }).to_string());
    let result = parse_frecency(&entries.join("\n"));
    assert_eq!(result.len(), MAX_FRECENCY_ENTRIES);
    assert_eq!(
        result[0],
        FrecencyEntry {
            path: "1000".to_string(),
            frequency: 2,
            last_open: 2000,
        }
    );
    assert!(!result.iter().any(|entry| entry.path == "0"));
}
