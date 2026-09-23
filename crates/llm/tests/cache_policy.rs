//! Port of packages/llm/test/cache-policy.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `applyCachePolicy` and the auto cache policy.
//! Effect-ts `Layer`/`Context` assertions are dropped; the wire-shape ones are kept.

use opencode_llm::{apply_cache_policy, Auth, CacheHint, LLMClient, Message, LLM};
use serde_json::json;

fn anthropic_model() -> serde_json::Value {
    json!({
        "id": "claude-sonnet-4-5",
        "provider": "anthropic",
        "route": { "id": "anthropic-messages" },
        "endpoint": { "baseURL": "https://api.anthropic.test/v1/" },
        "auth": Auth::header("x-api-key", "test"),
    })
}

#[test]
fn undefined_cache_resolves_to_auto() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "system": "You are concise.",
        "prompt": "hi",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["system"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert_eq!(
        prepared.body["messages"][0]["content"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
}

#[test]
fn auto_marks_the_last_tool_system_and_user_message_on_anthropic() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "system": "Sys A",
        "tools": [{ "name": "t1", "description": "t1", "inputSchema": { "type": "object", "properties": {} } }],
        "messages": [Message::user("first user"), Message::assistant("assistant reply"), Message::user("latest user message")],
        "cache": "auto",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["tools"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert_eq!(
        prepared.body["system"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert_eq!(
        prepared.body["messages"][2]["content"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert!(prepared.body["messages"][0]["content"][0]
        .get("cache_control")
        .is_none());
}

#[test]
fn auto_is_a_noop_on_openai_and_gemini() {
    let openai = LLMClient::prepare(LLM::request(json!({
        "model": { "id": "gpt-4o-mini", "provider": "openai", "route": { "id": "openai-chat" }, "endpoint": { "baseURL": "https://api.openai.test/v1/" }, "auth": Auth::bearer("test") },
        "system": "Sys",
        "prompt": "hi",
        "cache": "auto",
    })))
    .expect("prepare");
    let flat = openai.body.to_string();
    assert!(!flat.contains("cache_control"));
    assert!(!flat.contains("cachePoint"));

    let gemini = LLMClient::prepare(LLM::request(json!({
        "model": { "id": "gemini-2.5-flash", "provider": "google", "route": { "id": "gemini" }, "endpoint": { "baseURL": "https://generativelanguage.test/v1beta/" }, "auth": Auth::header("x-goog-api-key", "test") },
        "system": "Sys",
        "prompt": "hi",
        "cache": "auto",
    })))
    .expect("prepare");
    assert!(!gemini.body.to_string().contains("cache_control"));
}

#[test]
fn auto_on_bedrock_emits_cache_point_markers() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": { "id": "anthropic.claude-3-5-sonnet-20241022-v2:0", "provider": "amazon-bedrock", "route": { "id": "bedrock-converse" } },
        "system": "Sys",
        "tools": [{ "name": "t1", "description": "t1", "inputSchema": { "type": "object", "properties": {} } }],
        "messages": [Message::user("first user"), Message::assistant("reply"), Message::user("latest user")],
        "cache": "auto",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["toolConfig"]["tools"][1],
        json!({ "cachePoint": { "type": "default" } })
    );
    assert_eq!(
        prepared.body["system"][1],
        json!({ "cachePoint": { "type": "default" } })
    );
    assert_eq!(
        prepared.body["messages"][2]["content"][1],
        json!({ "cachePoint": { "type": "default" } })
    );
}

#[test]
fn none_disables_auto_placement_even_when_manual_hints_exist() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "system": "Sys",
        "tools": [{ "name": "t1", "description": "t1", "inputSchema": { "type": "object", "properties": {} } }],
        "prompt": "hi",
        "cache": "none",
    })))
    .expect("prepare");

    assert!(prepared.body["tools"][0].get("cache_control").is_none());
    assert!(prepared.body["system"][0].get("cache_control").is_none());
}

#[test]
fn granular_object_form_marks_just_tools() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "system": "Sys",
        "tools": [{ "name": "t1", "description": "t1", "inputSchema": { "type": "object", "properties": {} } }],
        "prompt": "hi",
        "cache": { "tools": true },
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["tools"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert!(prepared.body["system"][0].get("cache_control").is_none());
}

#[test]
fn auto_policy_preserves_manual_hints_on_other_parts() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "system": [
            { "type": "text", "text": "first system", "cache": CacheHint::new(json!({ "type": "ephemeral", "ttlSeconds": 3600 })) },
            { "type": "text", "text": "last system" }
        ],
        "prompt": "hi",
        "cache": "auto",
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["system"][0]["cache_control"],
        json!({ "type": "ephemeral", "ttl": "1h" })
    );
    assert_eq!(
        prepared.body["system"][1]["cache_control"],
        json!({ "type": "ephemeral" })
    );
}

#[test]
fn messages_tail_marks_the_last_two_message_boundaries() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "messages": [Message::user("u1"), Message::assistant("a1"), Message::user("u2"), Message::assistant("a2")],
        "cache": { "messages": { "tail": 2 } },
    })))
    .expect("prepare");

    assert!(prepared.body["messages"][0]["content"][0]
        .get("cache_control")
        .is_none());
    assert!(prepared.body["messages"][1]["content"][0]
        .get("cache_control")
        .is_none());
    assert_eq!(
        prepared.body["messages"][2]["content"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert_eq!(
        prepared.body["messages"][3]["content"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
}

#[test]
fn latest_assistant_marks_the_last_assistant_message() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": anthropic_model(),
        "messages": [Message::user("u1"), Message::assistant("a1"), Message::user("u2")],
        "cache": { "messages": "latest-assistant" },
    })))
    .expect("prepare");

    assert!(prepared.body["messages"][0]["content"][0]
        .get("cache_control")
        .is_none());
    assert_eq!(
        prepared.body["messages"][1]["content"][0]["cache_control"],
        json!({ "type": "ephemeral" })
    );
    assert!(prepared.body["messages"][2]["content"][0]
        .get("cache_control")
        .is_none());
}

#[test]
fn returns_the_same_request_reference_when_policy_is_a_noop() {
    let request =
        LLM::request(json!({ "model": anthropic_model(), "prompt": "hi", "cache": "none" }));
    let lowered = apply_cache_policy(request.clone()).expect("applyCachePolicy");

    assert_eq!(lowered, request);
}
