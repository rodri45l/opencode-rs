//! Port of packages/tui/test/prompt/part.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/prompt/part.ts; see docs/TEST-PORT.md.

use opencode_tui::prompt_display::prompt_offset_width;
use opencode_tui::prompt_part::{
    expand_tracked_pasted_text, strip_prompt_part_ids, PasteRange, PromptFileContent,
    PromptFilePart,
};

#[test]
fn strips_persisted_ids_from_reused_parts() {
    assert_eq!(
        strip_prompt_part_ids(PromptFilePart {
            id: "prt_old".to_string(),
            message_id: "msg_old".to_string(),
            session_id: "ses_old".to_string(),
            kind: "file".to_string(),
            mime: "image/png".to_string(),
            filename: "tiny.png".to_string(),
            url: "data:image/png;base64,abc".to_string(),
        }),
        PromptFileContent {
            kind: "file".to_string(),
            mime: "image/png".to_string(),
            filename: "tiny.png".to_string(),
            url: "data:image/png;base64,abc".to_string(),
        }
    );
}

#[test]
fn preserves_wide_characters_around_pasted_text() {
    let marker = "[Pasted ~3 lines]";
    let prefix = "你好你好\n";
    let text = format!("{prefix}{marker}\n阿斯顿法国红酒看来");
    let start = prompt_offset_width("你好你好") + 1;
    let end = start + prompt_offset_width(marker);

    assert_eq!(
        expand_tracked_pasted_text(
            &text,
            &[PasteRange {
                start,
                end,
                text: "public:\n\tvoid ExecuteTask();\nprivate:".to_string(),
            }],
        ),
        "你好你好\npublic:\n\tvoid ExecuteTask();\nprivate:\n阿斯顿法国红酒看来"
    );
}

#[test]
fn only_expands_the_tracked_placeholder_occurrence() {
    let marker = "[Pasted ~3 lines]";
    let prefix = format!("keep {marker} then ");
    let text = format!("{prefix}{marker} tail");
    let start = prompt_offset_width(&prefix);
    let end = start + prompt_offset_width(marker);

    assert_eq!(
        expand_tracked_pasted_text(
            &text,
            &[PasteRange {
                start,
                end,
                text: "alpha\nbeta\ngamma".to_string(),
            }],
        ),
        format!("keep {marker} then alpha\nbeta\ngamma tail")
    );
}
