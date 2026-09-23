//! Port of packages/tui/test/prompt/persistence.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/persistence.ts; see docs/TEST-PORT.md.

use opencode_tui::persistence::{append_text, read_json, read_text, write_json_atomic, write_text};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn persistence_creates_parent_directories_and_supports_text_append_and_json() {
    let root = std::env::temp_dir().join(format!(
        "opencode-tui-persistence-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let text_path = root.join("nested").join("state.jsonl");
    write_text(&text_path, "one\n").unwrap();
    append_text(&text_path, "two\n").unwrap();
    assert_eq!(read_text(&text_path).unwrap(), "one\ntwo\n");

    let json_path = root.join("other").join("state.json");
    write_json_atomic(&json_path, &serde_json::json!({ "value": 1 })).unwrap();
    assert_eq!(
        read_json::<serde_json::Value>(&json_path).unwrap(),
        serde_json::json!({ "value": 1 })
    );
    write_json_atomic(&json_path, &serde_json::json!({ "value": 2 })).unwrap();
    assert_eq!(
        read_json::<serde_json::Value>(&json_path).unwrap(),
        serde_json::json!({ "value": 2 })
    );

    let _ = std::fs::remove_dir_all(&root);
}
