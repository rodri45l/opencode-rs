//! Port of packages/opencode/test/session/message-v2.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `MessageV2.toModelMessages` user/assistant projection
//! (empty/ignored filtering, file and prompt injection, tool-call/result
//! pairing, truncation, compaction placeholders, aborts) and
//! `MessageV2.fromError` serialization.
//!
//! Dropped (provider-specific wire transforms): anthropic/bedrock media
//! re-placement, OpenRouter reasoning-details replay, signed-reasoning
//! empty-text substitution, and `latest`/`filterCompacted` ordering.

use opencode_server::port::message_v2::{from_error, to_model_messages};
use serde_json::{json, Value};

fn user(id: &str) -> Value {
    json!({
        "id": id,
        "sessionID": "session",
        "role": "user",
        "time": { "created": 0 },
        "agent": "user",
        "model": { "providerID": "test", "modelID": "test" },
        "tools": {},
        "mode": ""
    })
}

fn assistant(id: &str, parent: &str) -> Value {
    json!({
        "id": id,
        "sessionID": "session",
        "role": "assistant",
        "time": { "created": 0 },
        "parentID": parent,
        "modelID": "test-model",
        "providerID": "test",
        "mode": "",
        "agent": "agent"
    })
}

fn assistant_with_meta(id: &str, parent: &str, provider: &str, model: &str) -> Value {
    let mut info = assistant(id, parent);
    info["providerID"] = json!(provider);
    info["modelID"] = json!(model);
    info
}

fn project(input: Value, max: Option<usize>) -> Value {
    to_model_messages(&input, "test", "test-model", max)
}

#[test]
fn filters_out_messages_with_no_parts() {
    let input = json!([
        { "info": user("m-empty"), "parts": [] },
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "hello" }] }
    ]);
    assert_eq!(
        project(input, None),
        json!([{ "role": "user", "content": [{ "type": "text", "text": "hello" }] }])
    );
}

#[test]
fn filters_out_messages_with_only_ignored_parts() {
    let input = json!([
        {
            "info": user("m-user"),
            "parts": [{ "type": "text", "text": "ignored", "ignored": true }]
        }
    ]);
    assert_eq!(project(input, None), json!([]));
}

#[test]
fn filters_out_user_messages_with_only_empty_text_parts() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "" }] }
    ]);
    assert_eq!(project(input, None), json!([]));
}

#[test]
fn filters_empty_user_text_parts_while_keeping_non_empty() {
    let input = json!([
        {
            "info": user("m-user"),
            "parts": [
                { "type": "text", "text": "" },
                { "type": "text", "text": "hello" }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([{ "role": "user", "content": [{ "type": "text", "text": "hello" }] }])
    );
}

#[test]
fn includes_synthetic_text_parts() {
    let input = json!([
        {
            "info": user("m-user"),
            "parts": [{ "type": "text", "text": "hello", "synthetic": true }]
        },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [{ "type": "text", "text": "assistant", "synthetic": true }]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "hello" }] },
            { "role": "assistant", "content": [{ "type": "text", "text": "assistant" }] }
        ])
    );
}

#[test]
fn converts_user_text_and_file_parts_and_injects_prompts() {
    let input = json!([
        {
            "info": user("m-user"),
            "parts": [
                { "type": "text", "text": "hello" },
                { "type": "text", "text": "ignored", "ignored": true },
                { "type": "file", "mime": "image/png", "filename": "img.png", "url": "https://example.com/img.png" },
                { "type": "file", "mime": "text/plain", "filename": "note.txt", "url": "https://example.com/note.txt" },
                { "type": "file", "mime": "application/x-directory", "filename": "dir", "url": "https://example.com/dir" },
                { "type": "compaction", "auto": true },
                { "type": "subtask", "prompt": "prompt", "description": "desc", "agent": "agent" }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([{
            "role": "user",
            "content": [
                { "type": "text", "text": "hello" },
                {
                    "type": "file",
                    "mediaType": "image/png",
                    "filename": "img.png",
                    "data": "https://example.com/img.png"
                },
                { "type": "text", "text": "What did we do so far?" },
                { "type": "text", "text": "The following tool was executed by the user" }
            ]
        }])
    );
}

#[test]
fn converts_assistant_tool_completion_with_attachments() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [
                { "type": "text", "text": "done", "metadata": { "openai": { "assistant": "meta" } } },
                {
                    "type": "tool",
                    "callID": "call-1",
                    "tool": "bash",
                    "state": {
                        "status": "completed",
                        "input": { "cmd": "ls" },
                        "output": "ok",
                        "title": "Bash",
                        "metadata": {},
                        "time": { "start": 0, "end": 1 },
                        "attachments": [{
                            "type": "file",
                            "mime": "image/png",
                            "filename": "attachment.png",
                            "url": "data:image/png;base64,Zm9v"
                        }]
                    },
                    "metadata": { "openai": { "tool": "meta" } }
                }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "run tool" }] },
            {
                "role": "assistant",
                "content": [
                    { "type": "text", "text": "done", "providerOptions": { "openai": { "assistant": "meta" } } },
                    {
                        "type": "tool-call",
                        "toolCallId": "call-1",
                        "toolName": "bash",
                        "input": { "cmd": "ls" },
                        "providerOptions": { "openai": { "tool": "meta" } }
                    }
                ]
            },
            {
                "role": "tool",
                "content": [{
                    "type": "tool-result",
                    "toolCallId": "call-1",
                    "toolName": "bash",
                    "output": {
                        "type": "content",
                        "value": [
                            { "type": "text", "text": "ok" },
                            { "type": "media", "mediaType": "image/png", "data": "Zm9v" }
                        ]
                    },
                    "providerOptions": { "openai": { "tool": "meta" } }
                }]
            }
        ])
    );
}

#[test]
fn omits_provider_metadata_when_assistant_model_differs() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant_with_meta("m-assistant", "m-user", "other", "other"),
            "parts": [
                { "type": "text", "text": "done", "metadata": { "openai": { "assistant": "meta" } } },
                { "type": "reasoning", "text": "thinking", "metadata": { "openai": { "reasoning": "meta" } }, "time": { "start": 0 } },
                {
                    "type": "tool",
                    "callID": "call-1",
                    "tool": "bash",
                    "state": {
                        "status": "completed",
                        "input": { "cmd": "ls" },
                        "output": "ok",
                        "title": "Bash",
                        "metadata": {},
                        "time": { "start": 0, "end": 1 }
                    },
                    "metadata": { "openai": { "tool": "meta" } }
                }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "run tool" }] },
            {
                "role": "assistant",
                "content": [
                    { "type": "text", "text": "done" },
                    { "type": "text", "text": "thinking" },
                    { "type": "tool-call", "toolCallId": "call-1", "toolName": "bash", "input": { "cmd": "ls" } }
                ]
            },
            {
                "role": "tool",
                "content": [{
                    "type": "tool-result",
                    "toolCallId": "call-1",
                    "toolName": "bash",
                    "output": { "type": "text", "value": "ok" }
                }]
            }
        ])
    );
}

#[test]
fn replaces_compacted_tool_output_with_placeholder() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [{
                "type": "tool",
                "callID": "call-1",
                "tool": "bash",
                "state": {
                    "status": "completed",
                    "input": { "cmd": "ls" },
                    "output": "this should be cleared",
                    "title": "Bash",
                    "metadata": {},
                    "time": { "start": 0, "end": 1, "compacted": 1 }
                }
            }]
        }
    ]);
    let result = project(input, None);
    assert_eq!(
        result[2]["content"][0]["output"],
        json!({ "type": "text", "value": "[Old tool result content cleared]" })
    );
}

#[test]
fn truncates_tool_output_when_requested() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [{
                "type": "tool",
                "callID": "call-1",
                "tool": "bash",
                "state": {
                    "status": "completed",
                    "input": { "cmd": "ls" },
                    "output": "abcdefghij",
                    "title": "Shell",
                    "metadata": {},
                    "time": { "start": 0, "end": 1 }
                }
            }]
        }
    ]);
    let result = project(input, Some(4));
    assert_eq!(
        result[2]["content"][0]["output"],
        json!({
            "type": "text",
            "value": "abcd\n[Tool output truncated for compaction: omitted 6 chars]"
        })
    );
}

#[test]
fn converts_assistant_tool_error_into_error_text() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [{
                "type": "tool",
                "callID": "call-1",
                "tool": "bash",
                "state": {
                    "status": "error",
                    "input": { "cmd": "ls" },
                    "error": "nope",
                    "time": { "start": 0, "end": 1 },
                    "metadata": {}
                },
                "metadata": { "openai": { "tool": "meta" } }
            }]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "run tool" }] },
            {
                "role": "assistant",
                "content": [{
                    "type": "tool-call",
                    "toolCallId": "call-1",
                    "toolName": "bash",
                    "input": { "cmd": "ls" },
                    "providerOptions": { "openai": { "tool": "meta" } }
                }]
            },
            {
                "role": "tool",
                "content": [{
                    "type": "tool-result",
                    "toolCallId": "call-1",
                    "toolName": "bash",
                    "output": { "type": "error-text", "value": "nope" },
                    "providerOptions": { "openai": { "tool": "meta" } }
                }]
            }
        ])
    );
}

#[test]
fn forwards_partial_bash_output_for_aborted_tool_calls() {
    let output = [
        "31403",
        "12179",
        "4575",
        "",
        "<shell_metadata>",
        "User aborted the command",
        "</shell_metadata>",
    ]
    .join("\n");
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [{
                "type": "tool",
                "callID": "call-1",
                "tool": "bash",
                "state": {
                    "status": "error",
                    "input": { "command": "for i in {1..20}; do print -- $RANDOM; sleep 1; done" },
                    "error": "Tool execution aborted",
                    "metadata": { "interrupted": true, "output": output },
                    "time": { "start": 0, "end": 1 }
                }
            }]
        }
    ]);
    let result = project(input, None);
    assert_eq!(
        result[2]["content"][0]["output"],
        json!({ "type": "text", "value": output })
    );
}

#[test]
fn filters_assistant_messages_with_non_abort_errors() {
    let mut info = assistant("m-assistant", "m-parent");
    info["error"] =
        json!({ "name": "APIError", "data": { "message": "boom", "isRetryable": true } });
    let input = json!([
        { "info": info, "parts": [{ "type": "text", "text": "should not render" }] }
    ]);
    assert_eq!(project(input, None), json!([]));
}

#[test]
fn includes_aborted_assistant_messages_only_with_content() {
    let mut first = assistant("m-assistant-1", "m-parent");
    first["error"] = json!({ "name": "MessageAbortedError", "data": { "message": "aborted" } });
    let mut second = assistant("m-assistant-2", "m-parent");
    second["error"] = json!({ "name": "MessageAbortedError", "data": { "message": "aborted" } });
    let input = json!([
        {
            "info": first,
            "parts": [
                { "type": "reasoning", "text": "thinking", "time": { "start": 0 } },
                { "type": "text", "text": "partial answer" }
            ]
        },
        {
            "info": second,
            "parts": [
                { "type": "step-start" },
                { "type": "reasoning", "text": "thinking", "time": { "start": 0 } }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([{
            "role": "assistant",
            "content": [
                { "type": "reasoning", "text": "thinking" },
                { "type": "text", "text": "partial answer" }
            ]
        }])
    );
}

#[test]
fn splits_assistant_messages_on_step_start_boundaries() {
    let input = json!([
        {
            "info": assistant("m-assistant", "m-parent"),
            "parts": [
                { "type": "text", "text": "first" },
                { "type": "step-start" },
                { "type": "text", "text": "second" }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "assistant", "content": [{ "type": "text", "text": "first" }] },
            { "role": "assistant", "content": [{ "type": "text", "text": "second" }] }
        ])
    );
}

#[test]
fn drops_messages_that_only_contain_step_start_parts() {
    let input = json!([
        { "info": assistant("m-assistant", "m-parent"), "parts": [{ "type": "step-start" }] }
    ]);
    assert_eq!(project(input, None), json!([]));
}

#[test]
fn converts_pending_and_running_tool_calls_to_error_results() {
    let input = json!([
        { "info": user("m-user"), "parts": [{ "type": "text", "text": "run tool" }] },
        {
            "info": assistant("m-assistant", "m-user"),
            "parts": [
                {
                    "type": "tool",
                    "callID": "call-pending",
                    "tool": "bash",
                    "state": { "status": "pending", "input": { "cmd": "ls" }, "raw": "" }
                },
                {
                    "type": "tool",
                    "callID": "call-running",
                    "tool": "read",
                    "state": { "status": "running", "input": { "path": "/tmp" }, "time": { "start": 0 } }
                }
            ]
        }
    ]);
    assert_eq!(
        project(input, None),
        json!([
            { "role": "user", "content": [{ "type": "text", "text": "run tool" }] },
            {
                "role": "assistant",
                "content": [
                    { "type": "tool-call", "toolCallId": "call-pending", "toolName": "bash", "input": { "cmd": "ls" } },
                    { "type": "tool-call", "toolCallId": "call-running", "toolName": "read", "input": { "path": "/tmp" } }
                ]
            },
            {
                "role": "tool",
                "content": [
                    {
                        "type": "tool-result",
                        "toolCallId": "call-pending",
                        "toolName": "bash",
                        "output": { "type": "error-text", "value": "[Tool execution was interrupted]" }
                    },
                    {
                        "type": "tool-result",
                        "toolCallId": "call-running",
                        "toolName": "read",
                        "output": { "type": "error-text", "value": "[Tool execution was interrupted]" }
                    }
                ]
            }
        ])
    );
}

#[test]
fn serializes_context_length_exceeded_as_context_overflow() {
    let input = json!({ "type": "error", "error": { "code": "context_length_exceeded" } });
    assert_eq!(
        from_error(&input, "test"),
        json!({
            "name": "ContextOverflowError",
            "data": {
                "message": "Input exceeds context window of this model",
                "responseBody": serde_json::to_string(&input).unwrap()
            }
        })
    );
}

#[test]
fn serializes_response_error_codes() {
    let cases = [
        (
            "insufficient_quota",
            None,
            "Quota exceeded. Check your plan and billing details.",
        ),
        (
            "usage_not_included",
            None,
            "To use Codex with your ChatGPT plan, upgrade to Plus: https://chatgpt.com/explore/plus.",
        ),
        ("invalid_prompt", Some("Invalid prompt from test"), "Invalid prompt from test"),
    ];
    for (code, message, expected_message) in cases {
        let mut error = json!({ "code": code });
        if let Some(message) = message {
            error["message"] = json!(message);
        }
        let input = json!({ "type": "error", "error": error });
        assert_eq!(
            from_error(&input, "test"),
            json!({
                "name": "APIError",
                "data": {
                    "message": expected_message,
                    "isRetryable": false,
                    "responseBody": serde_json::to_string(&input).unwrap()
                }
            }),
            "code {code}"
        );
    }
}

#[test]
fn serializes_openai_server_error_stream_chunks_as_retryable() {
    let body = json!({
        "type": "error",
        "sequence_number": 2,
        "error": {
            "type": "server_error",
            "code": "server_error",
            "message": "An error occurred while processing your request.",
            "param": null
        }
    });
    let input = json!({ "message": serde_json::to_string(&body).unwrap() });
    assert_eq!(
        from_error(&input, "test"),
        json!({
            "name": "APIError",
            "data": {
                "message": "An error occurred while processing your request.",
                "isRetryable": true,
                "responseBody": serde_json::to_string(&body).unwrap()
            }
        })
    );
}

#[test]
fn detects_context_overflow_from_provider_messages() {
    let cases = [
        "prompt is too long: 213462 tokens > 200000 maximum",
        "Your input exceeds the context window of this model",
        "The input token count (1196265) exceeds the maximum number of tokens allowed (1048575)",
        "tokens in request more than max tokens allowed",
        "Please reduce the length of the messages or completion",
        "400 status code (no body)",
        "413 status code (no body)",
    ];
    for message in cases {
        let input = json!({
            "message": message,
            "statusCode": 400,
            "responseHeaders": { "content-type": "application/json" },
            "isRetryable": false
        });
        assert_eq!(
            from_error(&input, "test")["name"],
            json!("ContextOverflowError"),
            "message {message}"
        );
    }
}

#[test]
fn detects_context_overflow_from_response_body_code() {
    let input = json!({
        "message": "Request failed",
        "statusCode": 422,
        "responseBody": serde_json::to_string(&json!({
            "error": {
                "message": "Some message",
                "type": "invalid_request_error",
                "code": "context_length_exceeded"
            }
        }))
        .unwrap()
    });
    assert_eq!(
        from_error(&input, "test")["name"],
        json!("ContextOverflowError")
    );
}

#[test]
fn does_not_classify_429_without_body_as_context_overflow() {
    let input = json!({
        "message": "429 status code (no body)",
        "statusCode": 429,
        "responseHeaders": { "content-type": "application/json" },
        "isRetryable": false
    });
    assert_eq!(from_error(&input, "test")["name"], json!("APIError"));
}

#[test]
fn serializes_unknown_inputs() {
    assert_eq!(
        from_error(&json!(123), "test"),
        json!({ "name": "UnknownError", "data": { "message": "123" } })
    );
}

#[test]
fn classifies_zlib_errors() {
    let message = "ZlibError fetching \"https://opencode.cloudflare.dev/anthropic/messages\".";
    let input = json!({ "code": "ZlibError", "errno": 0, "path": "", "message": message });
    let result = from_error(&input, "test");
    assert_eq!(result["name"], json!("APIError"));
    assert_eq!(result["data"]["isRetryable"], json!(true));
    assert!(result["data"]["message"]
        .as_str()
        .unwrap()
        .contains("decompression"));

    let aborted = json!({ "code": "ZlibError", "errno": 0, "aborted": true, "message": message });
    assert_eq!(
        from_error(&aborted, "test")["name"],
        json!("MessageAbortedError")
    );
}
