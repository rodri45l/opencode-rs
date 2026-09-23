//! Port of packages/llm/test/prepare.test.ts (upstream 18ef3cc).
//! Behaviour pinned by provider-option merging and request precedence.

use opencode_llm::{Auth, LLMClient, LLM};
use serde_json::json;

#[test]
fn deep_merges_provider_option_records_and_replaces_arrays_primitives_and_null() {
    let merged = LLM::merge_provider_options(vec![
        json!({
            "openai": {
                "include": ["route"],
                "metadata": { "route": true, "shared": "route" },
                "nullable": "route",
                "primitive": "route"
            }
        }),
        json!({
            "openai": {
                "include": ["model"],
                "metadata": { "model": true, "shared": "model" },
                "nullable": null,
                "primitive": "model"
            }
        }),
        json!({ "openai": { "metadata": { "request": true }, "primitive": false } }),
    ])
    .expect("mergeProviderOptions");

    assert_eq!(
        merged,
        json!({
            "openai": {
                "include": ["model"],
                "metadata": { "route": true, "model": true, "request": true, "shared": "model" },
                "nullable": null,
                "primitive": false
            }
        })
    );
}

#[test]
fn prepares_bodies_with_route_model_and_call_options_in_order() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": {
            "id": "gpt-4o-mini",
            "provider": "openai",
            "route": { "id": "openai-chat" },
            "endpoint": { "baseURL": "https://api.openai.test/v1/" },
            "auth": Auth::bearer("test"),
            "defaults": {
                "generation": { "maxTokens": 20, "temperature": 0.5, "frequencyPenalty": 0.25, "stop": ["model"] },
                "providerOptions": { "openai": { "reasoningEffort": "medium" } }
            }
        },
        "prompt": "Say hello.",
        "generation": { "maxTokens": 30, "topP": 0.9, "stop": ["request"] },
        "providerOptions": { "openai": { "store": true } },
    })))
    .expect("prepare");

    assert_eq!(prepared.body["model"], "gpt-4o-mini");
    assert_eq!(prepared.body["stream"], true);
    assert_eq!(prepared.body["max_tokens"], 30);
    assert_eq!(prepared.body["temperature"], 0.5);
    assert_eq!(prepared.body["top_p"], 0.9);
    assert_eq!(prepared.body["frequency_penalty"], 0.25);
    assert_eq!(prepared.body["store"], true);
    assert_eq!(prepared.body["reasoning_effort"], "medium");
    assert_eq!(prepared.body["stop"], json!(["request"]));
}

#[test]
fn applies_model_http_defaults_before_request_http_overlays() {
    let prepared = LLMClient::prepare(LLM::request(json!({
        "model": {
            "id": "gpt-4o-mini",
            "provider": "openai",
            "route": { "id": "openai-chat" },
            "endpoint": { "baseURL": "https://api.openai.test/v1/" },
            "auth": Auth::bearer("fresh-key"),
            "defaults": { "http": { "query": { "model": "1", "shared": "model" } } }
        },
        "prompt": "Say hello.",
        "http": { "query": { "request": "1" } },
    })))
    .expect("prepare");

    assert_eq!(
        prepared.body["query"],
        json!({ "model": "1", "shared": "model", "request": "1" })
    );
}

#[test]
fn rejects_raw_body_overlays_for_protocol_owned_roots() {
    let error = LLMClient::prepare(LLM::request(json!({
        "model": {
            "id": "gpt-4o-mini",
            "provider": "openai",
            "route": { "id": "openai-chat" },
            "endpoint": { "baseURL": "https://api.openai.test/v1/" },
            "auth": Auth::bearer("test")
        },
        "prompt": "Say hello.",
        "http": { "body": { "model": "gpt-5", "messages": [], "tools": [] } },
    })))
    .expect_err("should reject");

    assert!(error
        .to_string()
        .contains("http.body cannot overlay protocol-owned field(s): model, messages, tools"));
}

#[test]
fn uses_model_output_limits_after_route_limits_and_before_call_max_tokens() {
    let base = json!({
        "model": {
            "id": "claude-sonnet-4-5",
            "provider": "anthropic",
            "route": { "id": "anthropic-messages" },
            "endpoint": { "baseURL": "https://api.anthropic.test/v1/" },
            "auth": Auth::header("x-api-key", "test"),
            "defaults": { "limits": { "output": 64 } }
        },
        "prompt": "Say hello.",
        "cache": "none",
    });

    let without = LLMClient::prepare(base.clone()).expect("prepare");
    let mut with = base;
    with["generation"] = json!({ "maxTokens": 32 });
    let with = LLMClient::prepare(with).expect("prepare");

    assert_eq!(without.body["max_tokens"], 64);
    assert_eq!(with.body["max_tokens"], 32);
}
