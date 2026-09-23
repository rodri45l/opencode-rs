//! Port of packages/opencode/test/plugin/cerebras.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Cerebras `chat.params` hook drops the generic output
//! cap when `max_completion_tokens` is configured, and never touches other
//! providers.

use opencode_server::port::plugin::{cerebras_chat_params, ChatParams};
use serde_json::json;

fn params(options: serde_json::Value) -> ChatParams {
    ChatParams {
        max_output_tokens: Some(32_000),
        options: options.as_object().cloned().unwrap_or_default(),
    }
}

#[test]
fn omits_the_generic_output_cap_when_max_completion_tokens_is_configured() {
    let mut output = params(json!({ "max_completion_tokens": 64 }));
    cerebras_chat_params("@ai-sdk/cerebras", &mut output);
    assert_eq!(output.max_output_tokens, None);
}

#[test]
fn preserves_the_generic_output_cap_without_max_completion_tokens() {
    let mut output = params(json!({}));
    cerebras_chat_params("@ai-sdk/cerebras", &mut output);
    assert_eq!(output.max_output_tokens, Some(32_000));
}

#[test]
fn does_not_change_other_providers() {
    let mut output = params(json!({ "max_completion_tokens": 64 }));
    cerebras_chat_params("@ai-sdk/openai", &mut output);
    assert_eq!(output.max_output_tokens, Some(32_000));
}
