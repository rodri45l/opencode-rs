//! Port of packages/core/test/github-copilot/openai-responses-language-model.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: generated item/response metadata is attached under the
//! `copilot` provider-metadata namespace (never `openai`); a tool call echoes a
//! stale `copilot.itemId` as the `function_call` id and omits it once stripped;
//! a reasoning part is preserved only with a `copilot.itemId` (otherwise dropped
//! with a `Non-OpenAI reasoning parts are not supported` warning); and a user
//! file part reads `imageDetail` from the `copilot` namespace.
//! Re-derived: the `LanguageModelV3` runtime, the mock fetch, and `doGenerate`
//! are replaced by the pure conversion helpers.

use opencode_core::github_copilot_responses::{
    convert_to_openai_responses_input, reasoning_metadata, ResponsesMetadata, REASONING_WARNING,
};
use serde_json::json;

const NOTE: &str = "porting: copilot responses conversion not implemented";

#[test]
#[ignore = "porting: copilot responses conversion not implemented"]
fn echoes_a_stale_tool_call_item_id_from_the_copilot_namespace() {
    let converted = convert_to_openai_responses_input(&json!([
        {
            "role": "assistant",
            "content": [
                {
                    "type": "tool-call",
                    "toolCallId": "call_1",
                    "toolName": "bash",
                    "input": { "command": "ls" },
                    "providerOptions": { "copilot": { "itemId": "fc_999" } }
                }
            ]
        }
    ]))
    .expect(NOTE);

    assert_eq!(
        converted.input,
        vec![json!({
            "type": "function_call",
            "call_id": "call_1",
            "name": "bash",
            "arguments": "{\"command\":\"ls\"}",
            "id": "fc_999"
        })]
    );
}

#[test]
#[ignore = "porting: copilot responses conversion not implemented"]
fn omits_the_function_call_id_once_the_stale_item_id_is_stripped() {
    let converted = convert_to_openai_responses_input(&json!([
        {
            "role": "assistant",
            "content": [
                {
                    "type": "tool-call",
                    "toolCallId": "call_1",
                    "toolName": "bash",
                    "input": { "command": "ls" },
                    "providerOptions": {}
                }
            ]
        }
    ]))
    .expect(NOTE);

    assert!(converted.input[0].get("id").is_none());
}

#[test]
#[ignore = "porting: copilot responses conversion not implemented"]
fn preserves_reasoning_keyed_by_the_copilot_namespace_and_drops_others() {
    let kept = convert_to_openai_responses_input(&json!([
        {
            "role": "assistant",
            "content": [
                {
                    "type": "reasoning",
                    "text": "thinking...",
                    "providerOptions": { "copilot": { "itemId": "rs_1", "reasoningEncryptedContent": "enc_1" } }
                }
            ]
        }
    ]))
    .expect(NOTE);
    assert!(kept.warnings.is_empty());
    assert_eq!(
        kept.input,
        vec![json!({
            "type": "reasoning",
            "id": "rs_1",
            "encrypted_content": "enc_1",
            "summary": [{ "type": "summary_text", "text": "thinking..." }]
        })]
    );

    let dropped = convert_to_openai_responses_input(&json!([
        { "role": "assistant", "content": [{ "type": "reasoning", "text": "thinking...", "providerOptions": {} }] }
    ]))
    .expect(NOTE);
    assert!(dropped.input.is_empty());
    assert_eq!(dropped.warnings.len(), 1);
    assert!(dropped.warnings[0]
        .as_str()
        .unwrap_or_default()
        .contains(REASONING_WARNING));
}

#[test]
#[ignore = "porting: copilot responses conversion not implemented"]
fn reads_image_detail_from_the_copilot_namespace_on_user_file_parts() {
    let converted = convert_to_openai_responses_input(&json!([
        {
            "role": "user",
            "content": [
                {
                    "type": "file",
                    "mediaType": "image/png",
                    "data": "aGVsbG8=",
                    "providerOptions": { "copilot": { "imageDetail": "high" } }
                }
            ]
        }
    ]))
    .expect(NOTE);

    assert_eq!(converted.input[0]["content"][0]["detail"], json!("high"));
}

#[test]
#[ignore = "porting: copilot responses conversion not implemented"]
fn attaches_generated_metadata_under_the_copilot_namespace_not_openai() {
    assert_eq!(
        ResponsesMetadata::reasoning("rs_1", Some("enc_1")).expect(NOTE),
        json!({ "copilot": { "itemId": "rs_1", "reasoningEncryptedContent": "enc_1" } })
    );
    assert_eq!(
        ResponsesMetadata::item("fc_1").expect(NOTE),
        json!({ "copilot": { "itemId": "fc_1" } })
    );
    assert_eq!(
        ResponsesMetadata::response("resp_1").expect(NOTE),
        json!({ "copilot": { "responseId": "resp_1" } })
    );
    assert!(reasoning_metadata("rs_1", None).get("openai").is_none());
}
