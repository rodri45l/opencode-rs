//! Port of packages/tui/test/prompt/history.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/prompt/history.tsx; see docs/TEST-PORT.md.

use opencode_tui::prompt_history::{
    is_duplicate_entry, parse_prompt_history, PromptInfo, PromptPartInfo, MAX_HISTORY_ENTRIES,
};

fn entry(input: &str, parts: Vec<PromptPartInfo>) -> PromptInfo {
    PromptInfo {
        input: input.to_string(),
        mode: None,
        parts,
    }
}

fn encoded(entry: &PromptInfo) -> String {
    serde_json::to_string(entry).unwrap()
}

#[test]
fn recovers_valid_jsonl_entries_around_corruption() {
    let text = format!(
        "{}\nnot-json\n{}\n",
        encoded(&entry("one", vec![])),
        encoded(&entry("two", vec![]))
    );
    assert_eq!(
        parse_prompt_history(&text),
        vec![entry("one", vec![]), entry("two", vec![])]
    );
}

#[test]
fn retains_only_the_newest_entries() {
    let input = (0..MAX_HISTORY_ENTRIES + 5)
        .map(|index| encoded(&entry(&index.to_string(), vec![])))
        .collect::<Vec<_>>()
        .join("\n");
    let result = parse_prompt_history(&input);
    assert_eq!(result.len(), MAX_HISTORY_ENTRIES);
    assert_eq!(result[0].input, "5");
}

#[test]
fn dedupes_only_identical_consecutive_entries() {
    assert!(!is_duplicate_entry(None, &entry("hello", vec![])));
    assert!(is_duplicate_entry(
        Some(&entry("hello", vec![])),
        &entry("hello", vec![])
    ));
    assert!(!is_duplicate_entry(
        Some(&entry("foo", vec![])),
        &entry("bar", vec![])
    ));
    let normal = PromptInfo {
        mode: Some("normal".to_string()),
        ..entry("ls", vec![])
    };
    let shell = PromptInfo {
        mode: Some("shell".to_string()),
        ..entry("ls", vec![])
    };
    assert!(!is_duplicate_entry(Some(&normal), &shell));
}

#[test]
fn does_not_dedupe_entries_with_different_parts() {
    let a = entry(
        "describe this",
        vec![PromptPartInfo::File {
            mime: "image/png".to_string(),
            filename: "a.png".to_string(),
            url: "data:image/png;base64,AAA".to_string(),
        }],
    );
    let b = entry(
        "describe this",
        vec![PromptPartInfo::File {
            mime: "image/png".to_string(),
            filename: "b.png".to_string(),
            url: "data:image/png;base64,BBB".to_string(),
        }],
    );
    assert!(!is_duplicate_entry(Some(&a), &b));
}
