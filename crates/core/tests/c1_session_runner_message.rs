//! Port of packages/core/test/session-runner-message.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `toLLMMessages` omits empty assistant turns, maps every
//! top-level V2 Session message type, replays durable tool media into canonical
//! tool messages without structured base64, restores OpenAI encrypted reasoning
//! metadata, and drops provider-native continuation metadata from failed assistant
//! turns and after a model switch.
//! Re-derived: the `@opencode-ai/llm` `Message`/`Model` types are represented as
//! JSON shapes; no protocol runtime is exercised.

#![allow(dead_code)]

use serde_json::{json, Value};

const NOTE: &str = "porting: session runner to-llm-message not implemented";

mod local {
    use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PortError {
        NotImplemented(&'static str),
    }

    pub fn to_llm_messages(_messages: &[Value], _model: &Value) -> Result<Vec<Value>, PortError> {
        Err(PortError::NotImplemented("session runner to-llm-message"))
    }
}

fn id(value: &str) -> String {
    format!("msg_{value}")
}

fn model() -> Value {
    json!({ "id": "model", "provider": "provider", "route": "openai-chat" })
}

fn assistant(id_value: &str, content: Value, model_ref: &Value) -> Value {
    json!({
        "id": id(id_value),
        "type": "assistant",
        "agent": "build",
        "model": model_ref,
        "content": content,
        "time": { "created": 0, "completed": 0 },
    })
}

fn file() -> Value {
    json!({ "uri": "data:image/png;base64,aGVsbG8=", "mime": "image/png", "name": "hello.png" })
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn omits_empty_assistant_turns() {
    let catalog_model = json!({ "id": "model", "providerID": "provider" });
    let messages = local::to_llm_messages(
        &[
            assistant("empty", json!([]), &catalog_model),
            assistant(
                "empty-text",
                json!([{ "type": "text", "id": "empty", "text": "" }]),
                &catalog_model,
            ),
            assistant(
                "empty-reasoning",
                json!([{ "type": "reasoning", "id": "empty-reasoning", "text": "" }]),
                &catalog_model,
            ),
            assistant(
                "text",
                json!([{ "type": "text", "id": "text", "text": "Partial" }]),
                &catalog_model,
            ),
            assistant(
                "reasoning",
                json!([{
                    "type": "reasoning",
                    "id": "reasoning",
                    "text": "",
                    "providerMetadata": { "anthropic": { "signature": "sig_1" } },
                }]),
                &catalog_model,
            ),
        ],
        &model(),
    )
    .expect(NOTE);

    let ids: Vec<&str> = messages
        .iter()
        .map(|message| message["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![id("text"), id("reasoning")]);
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn maps_every_top_level_v2_session_message_type() {
    let catalog_model = json!({ "id": "model", "providerID": "provider" });
    let messages = local::to_llm_messages(
        &[
            json!({ "id": id("agent"), "type": "agent-switched", "agent": "build", "time": { "created": 0 } }),
            json!({
                "id": id("model"),
                "type": "model-switched",
                "model": catalog_model,
                "time": { "created": 0 },
            }),
            json!({ "id": id("system"), "type": "system", "text": "Updated context\n\nOther context", "time": { "created": 0 } }),
            json!({
                "id": id("user"),
                "type": "user",
                "text": "Inspect this image",
                "files": [file()],
                "agents": [{ "name": "build" }],
                "time": { "created": 0 },
            }),
            json!({
                "id": id("synthetic"),
                "type": "synthetic",
                "sessionID": "ses_translate",
                "text": "Synthetic context",
                "time": { "created": 0 },
            }),
            json!({
                "id": id("shell"),
                "type": "shell",
                "callID": "shell-1",
                "command": "pwd",
                "output": "/project",
                "time": { "created": 0, "completed": 0 },
            }),
            json!({
                "id": id("compaction"),
                "type": "compaction",
                "reason": "auto",
                "summary": "Earlier work",
                "recent": "Recent work",
                "time": { "created": 0 },
            }),
        ],
        &model(),
    )
    .expect(NOTE);

    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().unwrap())
        .collect();
    assert_eq!(roles, vec!["system", "user", "user", "user", "user"]);
    assert_eq!(
        messages[0]["content"],
        json!("Updated context\n\nOther context")
    );
    assert_eq!(
        messages[1]["content"],
        json!([
            { "type": "text", "text": "Inspect this image" },
            { "type": "media", "mediaType": "image/png", "data": "data:image/png;base64,aGVsbG8=", "filename": "hello.png" },
        ])
    );
    assert_eq!(
        messages[1]["metadata"],
        json!({ "agents": [{ "name": "build" }] })
    );
    assert_eq!(
        messages[2]["content"],
        json!([{ "type": "text", "text": "Synthetic context" }])
    );
    assert_eq!(
        messages[3]["content"],
        json!([{ "type": "text", "text": "Shell command: pwd\n\n/project" }])
    );
    let checkpoint = messages[4]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(checkpoint.starts_with("<conversation-checkpoint>"));
    assert!(checkpoint.contains("<summary>\nEarlier work\n</summary>"));
    assert!(checkpoint.contains("<recent-context>\nRecent work\n</recent-context>"));
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn replays_durable_tool_media_into_canonical_tool_messages_without_structured_base64() {
    let catalog_model = json!({ "id": "model", "providerID": "provider" });
    let messages = local::to_llm_messages(
        &[assistant(
            "assistant",
            json!([
                { "type": "text", "id": "text-1", "text": "Checking" },
                {
                    "type": "reasoning",
                    "id": "reasoning-1",
                    "text": "Think",
                    "providerMetadata": { "anthropic": { "signature": "sig_1" } },
                },
                {
                    "type": "tool",
                    "id": "pending",
                    "name": "read",
                    "state": { "status": "pending", "input": "{\"path\":\"README.md\"}" },
                    "time": { "created": 0 },
                },
                {
                    "type": "tool",
                    "id": "running",
                    "name": "read",
                    "state": {
                        "status": "running",
                        "input": { "path": "README.md" },
                        "content": [],
                        "structured": { "type": "media", "mime": "image/png" },
                    },
                    "time": { "created": 0 },
                },
                {
                    "type": "tool",
                    "id": "completed",
                    "name": "read",
                    "state": {
                        "status": "completed",
                        "input": { "path": "README.md" },
                        "content": [{ "type": "text", "text": "Hello" }, file()],
                        "structured": {},
                    },
                    "time": { "created": 0, "completed": 0 },
                },
                {
                    "type": "tool",
                    "id": "hosted",
                    "name": "web_search",
                    "provider": {
                        "executed": true,
                        "metadata": { "fake": { "continuation": "hosted-call" } },
                        "resultMetadata": { "fake": { "continuation": "hosted-result" } },
                    },
                    "state": {
                        "status": "completed",
                        "input": { "query": "Effect" },
                        "content": [{ "type": "text", "text": "Found it" }],
                        "structured": {},
                    },
                    "time": { "created": 0, "completed": 0 },
                },
                {
                    "type": "tool",
                    "id": "hosted-failed",
                    "name": "write",
                    "provider": { "executed": true, "metadata": { "fake": { "continuation": "failed" } } },
                    "state": {
                        "status": "error",
                        "input": { "path": "README.md" },
                        "content": [],
                        "structured": {},
                        "error": { "type": "unknown", "message": "Denied" },
                    },
                    "time": { "created": 0, "completed": 0 },
                },
            ]),
            &catalog_model,
        )],
        &model(),
    )
    .expect(NOTE);

    let roles: Vec<&str> = messages
        .iter()
        .map(|message| message["role"].as_str().unwrap())
        .collect();
    assert_eq!(roles, vec!["assistant", "tool"]);
    assert_eq!(
        messages[0]["content"],
        json!([
            { "type": "text", "text": "Checking" },
            { "type": "reasoning", "text": "Think", "providerMetadata": { "anthropic": { "signature": "sig_1" } } },
            { "type": "tool-call", "id": "pending", "name": "read", "input": { "path": "README.md" } },
            { "type": "tool-call", "id": "running", "name": "read", "input": { "path": "README.md" } },
            { "type": "tool-call", "id": "completed", "name": "read", "input": { "path": "README.md" } },
            {
                "type": "tool-call",
                "id": "hosted",
                "name": "web_search",
                "input": { "query": "Effect" },
                "providerExecuted": true,
                "providerMetadata": { "fake": { "continuation": "hosted-call" } },
            },
            {
                "type": "tool-result",
                "id": "hosted",
                "name": "web_search",
                "providerExecuted": true,
                "providerMetadata": { "fake": { "continuation": "hosted-result" } },
                "result": { "type": "text", "value": "Found it" },
            },
            {
                "type": "tool-call",
                "id": "hosted-failed",
                "name": "write",
                "input": { "path": "README.md" },
                "providerExecuted": true,
                "providerMetadata": { "fake": { "continuation": "failed" } },
            },
            {
                "type": "tool-result",
                "id": "hosted-failed",
                "name": "write",
                "providerExecuted": true,
                "providerMetadata": { "fake": { "continuation": "failed" } },
                "result": {
                    "type": "error",
                    "value": { "error": { "type": "unknown", "message": "Denied" }, "content": [], "structured": {} },
                },
            },
        ])
    );
    assert_eq!(
        messages[1]["content"],
        json!([{
            "type": "tool-result",
            "id": "completed",
            "name": "read",
            "result": { "type": "content", "value": [{ "type": "text", "text": "Hello" }, file()] },
        }])
    );
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn restores_openai_encrypted_reasoning_metadata() {
    let catalog_model = json!({ "id": "model", "providerID": "provider" });
    let messages = local::to_llm_messages(
        &[assistant(
            "assistant-openai-reasoning",
            json!([{
                "type": "reasoning",
                "id": "reasoning-openai",
                "text": "Think",
                "providerMetadata": { "openai": { "itemId": "rs_1", "reasoningEncryptedContent": "encrypted-state" } },
            }]),
            &catalog_model,
        )],
        &model(),
    )
    .expect(NOTE);

    assert_eq!(
        messages[0]["content"],
        json!([{
            "type": "reasoning",
            "text": "Think",
            "providerMetadata": { "openai": { "itemId": "rs_1", "reasoningEncryptedContent": "encrypted-state" } },
        }])
    );
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn drops_provider_native_continuation_metadata_from_failed_assistant_turns() {
    let catalog_model = json!({ "id": "model", "providerID": "provider" });
    let mut failed = assistant(
        "assistant-failed",
        json!([
            {
                "type": "reasoning",
                "id": "reasoning-failed",
                "text": "Partial thought",
                "providerMetadata": { "openai": { "itemId": "rs_failed", "reasoningEncryptedContent": null } },
            },
            {
                "type": "tool",
                "id": "hosted-failed",
                "name": "web_search",
                "provider": {
                    "executed": true,
                    "metadata": { "openai": { "itemId": "call_failed" } },
                    "resultMetadata": { "openai": { "itemId": "result_failed" } },
                },
                "state": {
                    "status": "error",
                    "input": { "query": "Effect" },
                    "error": { "type": "unknown", "message": "Provider turn interrupted" },
                    "content": [],
                    "structured": {},
                },
                "time": { "created": 0, "completed": 0 },
            },
        ]),
        &catalog_model,
    );
    failed["finish"] = json!("error");
    failed["error"] = json!({ "type": "unknown", "message": "Provider turn interrupted" });

    let messages = local::to_llm_messages(&[failed], &model()).expect(NOTE);
    let content = &messages[0]["content"];
    assert!(content[0].get("providerMetadata").is_none());
    assert!(content[1].get("providerMetadata").is_none());
    assert!(content[2].get("providerMetadata").is_none());
    assert_eq!(content[2]["result"]["type"], json!("error"));
}

#[test]
#[ignore = "porting: session runner to-llm-message not implemented"]
fn drops_provider_native_continuation_metadata_after_a_model_switch() {
    let old_model = json!({ "id": "old-model", "providerID": "provider" });
    let messages = local::to_llm_messages(
        &[assistant(
            "assistant-old-model",
            json!([
                {
                    "type": "reasoning",
                    "id": "reasoning-old-model",
                    "text": "Visible thought",
                    "providerMetadata": { "anthropic": { "signature": "sig_old" } },
                },
                {
                    "type": "tool",
                    "id": "hosted-old-model",
                    "name": "web_search",
                    "provider": {
                        "executed": true,
                        "metadata": { "openai": { "itemId": "hosted-old-model" } },
                        "resultMetadata": { "openai": { "itemId": "hosted-old-model" } },
                    },
                    "state": {
                        "status": "completed",
                        "input": { "query": "Effect" },
                        "content": [],
                        "structured": {},
                        "result": { "type": "json", "value": { "status": "completed" } },
                    },
                    "time": { "created": 0, "completed": 0 },
                },
                {
                    "type": "tool",
                    "id": "local-old-model",
                    "name": "read",
                    "provider": {
                        "executed": false,
                        "metadata": { "fake": { "call": "old" } },
                        "resultMetadata": { "fake": { "result": "old" } },
                    },
                    "state": {
                        "status": "completed",
                        "input": { "path": "README.md" },
                        "content": [],
                        "structured": { "text": "Hello" },
                    },
                    "time": { "created": 0, "completed": 0 },
                },
            ]),
            &old_model,
        )],
        &model(),
    )
    .expect(NOTE);

    let content = &messages[0]["content"];
    assert_eq!(
        content[0],
        json!({ "type": "text", "text": "Visible thought" })
    );
    assert_eq!(content[1]["type"], json!("tool-call"));
    assert!(content[1].get("providerMetadata").is_none());
    assert_eq!(
        content[2]["result"],
        json!({ "type": "json", "value": { "status": "completed" } })
    );
    assert_eq!(content[3]["providerExecuted"], json!(false));
    assert_eq!(
        messages[1]["content"][0]["result"],
        json!({ "type": "json", "value": { "text": "Hello" } })
    );
}
