//! Port of packages/core/test/shared-schema.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: shared record schemas construct and decode to plain values,
//! prompt equality is structural, a skill directory source keys as
//! `directory:<path>`, and workspace ids ascend with a `wrk_` prefix.
//! Re-derived: the first upstream test asserts that core and `@opencode-ai/schema`
//! export the *same* objects; Rust has a single schema crate, so that identity
//! check is noted rather than ported.

use opencode_core::shared_schema::{AssistantText, Prompt, SkillSource, WorkspaceId};
use serde_json::json;

const NOTE: &str = "porting: shared schema not implemented";

#[test]
#[ignore = "porting: shared schema not implemented"]
fn shared_record_schemas_construct_and_decode_plain_objects() {
    let made = Prompt::make("hello").expect(NOTE);
    let decoded = Prompt::decode(&json!({ "text": "hello" })).expect(NOTE);
    let content = AssistantText::decode(&json!({ "type": "text", "id": "part_1", "text": "hi" }))
        .expect(NOTE);

    assert_eq!(
        made,
        Prompt {
            text: "hello".into()
        }
    );
    assert_eq!(made, decoded);
    assert_eq!(
        content,
        AssistantText {
            id: "part_1".into(),
            text: "hi".into()
        }
    );
    assert_eq!(Prompt::from_user_message("hello").expect(NOTE), made);
}

#[test]
#[ignore = "porting: shared schema not implemented"]
fn skill_sources_key_by_type_and_path() {
    assert_eq!(
        SkillSource::key(&json!({ "type": "directory", "path": "/tmp" })).expect(NOTE),
        "directory:/tmp"
    );
}

#[test]
#[ignore = "porting: shared schema not implemented"]
fn workspace_ids_ascend_with_the_expected_prefix() {
    assert!(WorkspaceId::ascending("").expect(NOTE).starts_with("wrk_"));
}
