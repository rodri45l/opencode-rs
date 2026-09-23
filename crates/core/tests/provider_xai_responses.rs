//! Port of packages/core/test/provider-xai-responses.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the xAI responses API lowers `promptCacheKey` to
//! `prompt_cache_key` and `reasoningEffort` to `reasoning.effort`, while the chat
//! API lowers `reasoningEffort` to a flat `reasoning_effort` (including `xhigh`).
//! Re-derived: the `@ai-sdk/xai` models and mock fetch are replaced by direct
//! body lowering.

use opencode_core::xai_responses::{XaiApi, XaiResponsesPlugin};
use serde_json::json;

#[test]
fn responses_sends_prompt_cache_key_and_nested_reasoning() {
    let mut body = json!({});
    XaiResponsesPlugin::apply_options(
        XaiApi::Responses,
        &json!({ "xai": { "promptCacheKey": "session-123" } }),
        &mut body,
    )
    .unwrap();
    assert_eq!(body["prompt_cache_key"], json!("session-123"));

    let mut body = json!({});
    XaiResponsesPlugin::apply_options(
        XaiApi::Responses,
        &json!({ "xai": { "reasoningEffort": "xhigh" } }),
        &mut body,
    )
    .unwrap();
    assert_eq!(body["reasoning"], json!({ "effort": "xhigh" }));
}

#[test]
fn chat_sends_flat_reasoning_effort() {
    let mut body = json!({});
    XaiResponsesPlugin::apply_options(
        XaiApi::Chat,
        &json!({ "xai": { "reasoningEffort": "xhigh" } }),
        &mut body,
    )
    .unwrap();
    assert_eq!(body["reasoning_effort"], json!("xhigh"));
}
