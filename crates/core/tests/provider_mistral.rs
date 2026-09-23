//! Port of packages/core/test/provider-mistral.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `promptCacheKey` lowers to `prompt_cache_key`,
//! `reasoningEffort` lowers to `reasoning_effort` (including unknown values), a
//! metadata-only thinking part survives normalization, and a plain reasoning part
//! collapses with adjacent text in assistant history. Re-derived: the
//! `@ai-sdk/mistral` model, mock fetch, and streaming cases are replaced by
//! direct lowering helpers; the streaming metadata test is noted as dropped.

use opencode_core::mistral::MistralPlugin;
use serde_json::json;

const NOTE: &str = "porting: mistral provider lowering not implemented";

#[test]
#[ignore = "porting: mistral provider lowering not implemented"]
fn sends_prompt_cache_key_and_reasoning_effort() {
    let mut body = json!({});
    MistralPlugin::apply_request(
        &json!({ "mistral": { "promptCacheKey": "session-123" } }),
        &mut body,
    )
    .expect(NOTE);
    assert_eq!(body["prompt_cache_key"], json!("session-123"));

    let mut body = json!({});
    MistralPlugin::apply_request(
        &json!({ "mistral": { "reasoningEffort": "custom" } }),
        &mut body,
    )
    .expect(NOTE);
    assert_eq!(body["reasoning_effort"], json!("custom"));
}

#[test]
#[ignore = "porting: mistral provider lowering not implemented"]
fn preserves_metadata_only_thinking_chunks() {
    let thinking = json!({
        "type": "thinking",
        "thinking": [
            {
                "type": "tool_reference",
                "tool": "web_search",
                "title": "Example result",
                "url": "https://example.com/tool",
                "favicon": "https://example.com/favicon.ico",
                "description": "Example description"
            },
            { "type": "reference", "reference_ids": [1, "source-2"] }
        ],
        "closed": true,
        "signature": "sig-123"
    });

    assert_eq!(
        MistralPlugin::normalize_thinking(&thinking).expect(NOTE),
        json!({
            "type": "reasoning",
            "text": "",
            "providerMetadata": { "mistral": { "thinking": thinking } }
        })
    );
}

#[test]
#[ignore = "porting: mistral provider lowering not implemented"]
fn collapses_plain_reasoning_with_adjacent_text_in_history() {
    let content = json!([
        { "type": "reasoning", "text": "thinking" },
        { "type": "text", "text": "Hi" }
    ]);
    assert_eq!(
        MistralPlugin::history_content(&content).expect(NOTE),
        json!("thinkingHi")
    );
}
