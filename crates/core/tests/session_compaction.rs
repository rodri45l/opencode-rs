//! Port of packages/core/test/session-compaction.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the compaction prompt embeds the conversation (and any prior
//! summary) and asks for an anchored summary with a fixed work-state shape; tool
//! media is described without embedding base64.

use opencode_core::session_compaction::SessionCompaction;
use serde_json::json;

const NOTE: &str = "porting: session compaction not implemented";

#[test]
fn compaction_prompt_preserves_detailed_work_state_and_relevant_files() {
    let prompt =
        SessionCompaction::build_prompt(&["conversation history".to_string()], None).expect(NOTE);

    assert!(prompt.starts_with(
        "Here is the conversation so far:\n\n<conversation>\nconversation history\n</conversation>"
    ));
    assert!(
        prompt.find("</conversation>").unwrap()
            < prompt.find("Create a new anchored summary").unwrap()
    );
    assert!(prompt.contains("conversation history in the <conversation> tags above"));
    assert!(prompt.contains("## Work State\n### Completed"));
    assert!(prompt.contains("### Active"));
    assert!(prompt.contains("### Blocked"));
    assert!(prompt.contains("## Relevant Files"));
}

#[test]
fn compaction_prompt_gives_update_instructions_for_a_prior_summary() {
    let prompt = SessionCompaction::build_prompt(
        &["new conversation".to_string()],
        Some("existing summary"),
    )
    .expect(NOTE);

    assert!(prompt.find("<conversation>").unwrap() < prompt.find("<prior-summary>").unwrap());
    assert!(
        prompt.find("</prior-summary>").unwrap()
            < prompt.find("The <prior-summary> summarizes").unwrap()
    );
    assert!(prompt.contains(
        "Carry forward objectives, constraints, user directives, decisions, and parallel workstreams from the <prior-summary>"
    ));
    assert!(prompt.contains("Move completed work from \"Active\" to \"Completed\"."));
    assert!(prompt
        .contains("Update \"Objective\" and \"Next Move\" to reflect the current work state."));
}

#[test]
fn compaction_describes_tool_media_without_embedding_base64() {
    let base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB";
    let serialized = SessionCompaction::serialize_tool_content(&[
        json!({ "type": "text", "text": "Image read successfully" }),
        json!({
            "type": "file",
            "uri": format!("data:image/png;base64,{base64}"),
            "mime": "image/png",
            "name": "pixel.png",
        }),
    ])
    .expect(NOTE);

    assert_eq!(
        serialized,
        "Image read successfully\n[Attached image/png: pixel.png]"
    );
    assert!(!serialized.contains(base64));
}
