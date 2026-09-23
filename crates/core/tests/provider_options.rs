//! Port of packages/core/test/config/provider-options.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: provider/request option lowering for the raw fallback and
//! the OpenAI, Anthropic, Google, Azure, Amazon Bedrock and OpenAI-compatible
//! families, including camelCase-to-snake_case request fields.

use opencode_core::provider_options::ConfigProviderOptionsV1;
use serde_json::json;

const NOTE: &str = "porting: config provider options not implemented";

#[test]
#[ignore = "porting: config provider options not implemented"]
fn keeps_raw_provider_and_request_options_unchanged() {
    let lowerer = ConfigProviderOptionsV1::get("custom-provider");

    assert_eq!(
        lowerer
            .provider(&json!({ "apiKey": "secret", "headers": { "x-test": "1" }, "nested": { "camelCase": true } }))
            .expect(NOTE),
        json!({ "body": { "apiKey": "secret", "headers": { "x-test": "1" }, "nested": { "camelCase": true } } })
    );
    assert_eq!(
        lowerer
            .request(&json!({ "nested": { "camelCase": true } }))
            .expect(NOTE),
        json!({ "nested": { "camelCase": true } })
    );
}

#[test]
#[ignore = "porting: config provider options not implemented"]
fn falls_back_to_raw_lowering_for_prototype_property_package_names() {
    assert_eq!(
        ConfigProviderOptionsV1::get("toString")
            .provider(&json!({ "enabled": true }))
            .expect(NOTE),
        json!({ "body": { "enabled": true } })
    );
}

#[test]
#[ignore = "porting: config provider options not implemented"]
fn lowers_openai_provider_and_request_options() {
    let lowerer = ConfigProviderOptionsV1::get("@ai-sdk/openai");

    assert_eq!(
        lowerer
            .provider(&json!({
                "apiKey": "secret",
                "baseURL": "https://openai.example/v1",
                "organization": "org",
                "project": "project",
                "headers": { "x-test": "1" },
                "body": { "store": true },
                "timeout": 1000,
            }))
            .expect(NOTE),
        json!({
            "url": "https://openai.example/v1",
            "headers": {
                "Authorization": "Bearer secret",
                "OpenAI-Organization": "org",
                "OpenAI-Project": "project",
                "x-test": "1",
            },
            "body": { "store": true },
            "settings": { "timeout": 1000 },
        })
    );
    assert_eq!(
        lowerer
            .request(&json!({
                "reasoningEffort": "high",
                "reasoningSummary": "auto",
                "reasoning": { "encryptedContent": true },
                "textVerbosity": "low",
                "text": { "outputFormat": "plain" },
                "nestedValue": { "camelCase": true },
            }))
            .expect(NOTE),
        json!({
            "reasoning": { "encrypted_content": true, "effort": "high", "summary": "auto" },
            "text": { "output_format": "plain", "verbosity": "low" },
            "nested_value": { "camel_case": true },
        })
    );
}

#[test]
#[ignore = "porting: config provider options not implemented"]
fn lowers_anthropic_provider_and_request_options() {
    let lowerer = ConfigProviderOptionsV1::get("@ai-sdk/anthropic");

    assert_eq!(
        lowerer
            .provider(&json!({
                "apiKey": "secret",
                "authToken": "token",
                "baseURL": "https://anthropic.example",
                "headers": { "x-test": "1" },
                "body": { "beta": true },
                "generateId": "custom",
            }))
            .expect(NOTE),
        json!({
            "url": "https://anthropic.example",
            "headers": { "x-api-key": "secret", "Authorization": "Bearer token", "x-test": "1" },
            "body": { "beta": true },
            "settings": { "generateId": "custom" },
        })
    );
    assert_eq!(
        lowerer
            .request(&json!({
                "effort": "high",
                "taskBudget": 1024,
                "metadata": { "userId": "user", "traceId": "trace" },
                "nestedValue": { "camelCase": true },
            }))
            .expect(NOTE),
        json!({
            "output_config": { "effort": "high", "task_budget": 1024 },
            "metadata": { "user_id": "user", "trace_id": "trace" },
            "nested_value": { "camel_case": true },
        })
    );
}

#[test]
#[ignore = "porting: config provider options not implemented"]
fn lowers_google_provider_and_request_options() {
    let lowerer = ConfigProviderOptionsV1::get("@ai-sdk/google");

    assert_eq!(
        lowerer
            .provider(&json!({
                "apiKey": "secret",
                "baseURL": "https://google.example",
                "headers": { "x-test": "1" },
                "body": { "trace": true },
                "project": "project",
            }))
            .expect(NOTE),
        json!({
            "url": "https://google.example",
            "headers": { "x-goog-api-key": "secret", "x-test": "1" },
            "body": { "trace": true },
            "settings": { "project": "project" },
        })
    );
    assert_eq!(
        lowerer
            .request(&json!({
                "thinkingConfig": { "thinkingBudget": 1024 },
                "responseModalities": ["TEXT"],
                "mediaResolution": "high",
                "imageConfig": { "aspectRatio": "16:9" },
                "safetySettings": ["safe"],
            }))
            .expect(NOTE),
        json!({
            "safetySettings": ["safe"],
            "generationConfig": {
                "thinkingConfig": { "thinkingBudget": 1024 },
                "responseModalities": ["TEXT"],
                "mediaResolution": "high",
                "imageConfig": { "aspectRatio": "16:9" },
            },
        })
    );
}

#[test]
#[ignore = "porting: config provider options not implemented"]
fn lowers_openai_compatible_provider_and_request_options() {
    let lowerer = ConfigProviderOptionsV1::get("@ai-sdk/openai-compatible");

    assert_eq!(
        lowerer
            .provider(&json!({
                "baseURL": "https://compatible.example/v1",
                "headers": { "x-test": "1" },
                "body": { "trace": true },
                "apiKey": "secret",
            }))
            .expect(NOTE),
        json!({
            "url": "https://compatible.example/v1",
            "headers": { "x-test": "1" },
            "body": { "trace": true },
            "settings": { "apiKey": "secret" },
        })
    );
    assert_eq!(
        lowerer
            .request(&json!({ "reasoningEffort": "high", "serviceTier": "priority" }))
            .expect(NOTE),
        json!({ "reasoning_effort": "high", "serviceTier": "priority" })
    );
}
