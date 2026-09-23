//! Port of packages/core/test/github-copilot/convert-to-copilot-messages.test.ts
//! (upstream 18ef3cc).
//!
//! Behaviour pinned: canonical prompt messages convert to OpenAI-compatible chat
//! messages — system text, user text/image parts, assistant text and tool calls,
//! tool results, and copilot reasoning fields (`reasoning_text` only alongside
//! `reasoning_opaque`). Dropped: the `Uint8Array` image input case, which is
//! folded into the base64 representation.

use opencode_core::copilot_convert::convert_to_openai_compatible_chat_messages;
use serde_json::json;

const NOTE: &str = "porting: copilot message conversion not implemented";

fn convert(messages: serde_json::Value) -> serde_json::Value {
    let messages = messages.as_array().expect("array").clone();
    serde_json::Value::Array(convert_to_openai_compatible_chat_messages(&messages).expect(NOTE))
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn converts_system_message_content_to_string() {
    assert_eq!(
        convert(json!([
            { "role": "system", "content": "You are a helpful assistant with AGENTS.md instructions." }
        ])),
        json!([
            { "role": "system", "content": "You are a helpful assistant with AGENTS.md instructions." }
        ])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn converts_a_text_only_user_message_to_string_content() {
    assert_eq!(
        convert(json!([{ "role": "user", "content": [{ "type": "text", "text": "Hello" }] }])),
        json!([{ "role": "user", "content": "Hello" }])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn converts_user_image_parts() {
    assert_eq!(
        convert(json!([
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": "Hello" },
                    { "type": "file", "data": "AAECAw==", "mediaType": "image/png" }
                ]
            }
        ])),
        json!([
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": "Hello" },
                    { "type": "image_url", "image_url": { "url": "data:image/png;base64,AAECAw==" } }
                ]
            }
        ])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn handles_url_based_images() {
    assert_eq!(
        convert(json!([
            {
                "role": "user",
                "content": [{ "type": "file", "data": "https://example.com/image.jpg", "mediaType": "image/*" }]
            }
        ])),
        json!([
            {
                "role": "user",
                "content": [{ "type": "image_url", "image_url": { "url": "https://example.com/image.jpg" } }]
            }
        ])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn handles_multiple_text_parts_without_flattening() {
    assert_eq!(
        convert(json!([
            {
                "role": "user",
                "content": [{ "type": "text", "text": "Part 1" }, { "type": "text", "text": "Part 2" }]
            }
        ])),
        json!([
            {
                "role": "user",
                "content": [{ "type": "text", "text": "Part 1" }, { "type": "text", "text": "Part 2" }]
            }
        ])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn converts_assistant_text_messages() {
    assert_eq!(
        convert(
            json!([{ "role": "assistant", "content": [{ "type": "text", "text": "Hello back!" }] }])
        ),
        json!([{ "role": "assistant", "content": "Hello back!" }])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn handles_assistant_tool_calls_with_null_content() {
    assert_eq!(
        convert(json!([
            {
                "role": "assistant",
                "content": [{ "type": "tool-call", "toolCallId": "call1", "toolName": "calculator", "input": { "a": 1, "b": 2 } }]
            }
        ])),
        json!([
            {
                "role": "assistant",
                "content": null,
                "tool_calls": [
                    {
                        "id": "call1",
                        "type": "function",
                        "function": { "name": "calculator", "arguments": "{\"a\":1,\"b\":2}" }
                    }
                ]
            }
        ])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn converts_text_tool_results() {
    assert_eq!(
        convert(json!([
            {
                "role": "tool",
                "content": [
                    { "type": "tool-result", "toolCallId": "call-1", "toolName": "getWeather", "output": { "type": "text", "value": "It is sunny today" } }
                ]
            }
        ])),
        json!([{ "role": "tool", "tool_call_id": "call-1", "content": "It is sunny today" }])
    );
}

#[test]
#[ignore = "porting: copilot message conversion not implemented"]
fn includes_reasoning_opaque_from_provider_options() {
    assert_eq!(
        convert(json!([
            {
                "role": "assistant",
                "content": [
                    { "type": "reasoning", "text": "Thinking...", "providerOptions": { "copilot": { "reasoningOpaque": "opaque-signature-123" } } },
                    { "type": "text", "text": "Done!" }
                ]
            }
        ])),
        json!([
            {
                "role": "assistant",
                "content": "Done!",
                "reasoning_text": "Thinking...",
                "reasoning_opaque": "opaque-signature-123"
            }
        ])
    );
}
