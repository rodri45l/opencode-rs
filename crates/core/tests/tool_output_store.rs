//! Port of packages/core/test/tool-output-store.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: oversized text is written to one managed file and replaced
//! with a bounded preview, structured-only output is bounded via JSON, native
//! media is preserved without a settlement limit, and duplicated structured data
//! is not double-counted. Dropped: the interruption and config-layer cases.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::tool_output_store::{ToolOutput, ToolOutputStore};
use serde_json::json;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-tooloutput-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn store() -> ToolOutputStore {
    ToolOutputStore::with_root(scratch())
}

#[test]
fn bounds_the_provider_facing_text_channel_with_one_managed_file() {
    let store = store();
    let first = format!("HEAD-{}", "x".repeat(30_000));
    let second = format!("{}-TAIL", "y".repeat(30_000));

    let result = store
        .bound(
            "ses_tool_output_store",
            "call-aggregate",
            ToolOutput {
                structured: json!({ "kind": "report" }),
                content: vec![
                    json!({ "type": "text", "text": first }),
                    json!({ "type": "text", "text": second }),
                ],
            },
        )
        .unwrap();

    assert_eq!(result.output.structured, json!({ "kind": "report" }));
    assert_eq!(result.output_paths.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&result.output_paths[0]).unwrap(),
        format!("{first}{second}")
    );
    let preview = result.output.content[0]["text"].as_str().unwrap();
    assert!(preview.len() <= ToolOutputStore::MAX_BYTES);
}

#[test]
fn uses_bounded_text_for_oversized_structured_only_output() {
    let store = store();
    let structured = json!({ "text": "x".repeat(ToolOutputStore::MAX_BYTES) });

    let result = store
        .bound(
            "ses_tool_output_store",
            "call-json",
            ToolOutput {
                structured: structured.clone(),
                content: vec![],
            },
        )
        .unwrap();

    assert_eq!(result.output.structured, structured);
    assert_eq!(result.output_paths.len(), 1);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&result.output_paths[0]).unwrap()
        )
        .unwrap(),
        structured
    );
    assert_eq!(result.output.content.len(), 1);
}

#[test]
fn preserves_native_media_without_applying_a_settlement_media_limit() {
    let store = store();
    let data = "a".repeat(6 * 1024 * 1024);
    let media = json!({
        "type": "file",
        "uri": format!("data:image/png;base64,{data}"),
        "mime": "image/png",
        "name": "pixel.png",
    });

    let result = store
        .bound(
            "ses_tool_output_store",
            "call-file",
            ToolOutput {
                structured: json!({ "caption": "pixel" }),
                content: vec![media.clone()],
            },
        )
        .unwrap();

    assert_eq!(result.output_paths, Vec::<PathBuf>::new());
    assert_eq!(result.output.structured, json!({ "caption": "pixel" }));
    assert_eq!(result.output.content, vec![media]);
}

#[test]
fn does_not_double_count_structured_data_duplicated_in_projected_text() {
    let store = store();
    let text = "x".repeat(30_000);
    let output = ToolOutput {
        structured: json!({ "output": text }),
        content: vec![json!({ "type": "text", "text": text })],
    };

    let result = store
        .bound("ses_tool_output_store", "call-duplicated", output.clone())
        .unwrap();

    assert_eq!(result.output, output);
    assert_eq!(result.output_paths, Vec::<PathBuf>::new());
}
