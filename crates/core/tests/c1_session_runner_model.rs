//! Port of packages/core/test/session-runner-model.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: catalog AI-SDK models map to native route ids, `apiKey`
//! credentials never reach the provider JSON, merged API settings drive bearer
//! auth, selected Session variants overlay route headers/body, an unavailable
//! explicit variant and an unsupported API fail with their exact messages, stored
//! credentials win over configured auth, OAuth metadata is not projected, and
//! `supported` reports whether a native route exists.
//! Re-derived: `LLMClient.prepare`/`route.auth.apply` are replaced by reading the
//! resolved route auth token and http body directly.

#![allow(dead_code)]

use serde_json::{json, Value};

const NOTE: &str = "porting: session runner model resolution not implemented";

use opencode_core::session_runner_model::{
    from_catalog_model, resolve, supported, unsupported_api_message, variant_unavailable_message,
};

fn model(api: Value, variants: Value) -> Value {
    json!({
        "id": "test-model",
        "providerID": "test-provider",
        "name": "Test model",
        "api": {
            "id": "api-test-model",
            "type": api["type"],
            "package": api["package"],
            "url": api["url"],
            "settings": api.get("settings").cloned().unwrap_or(Value::Null),
        },
        "capabilities": { "tools": true, "input": ["text"], "output": ["text"] },
        "request": {
            "headers": { "x-test": "header" },
            "body": { "apiKey": "secret", "custom_extension": { "enabled": true } },
        },
        "variants": variants,
        "time": { "released": 0 },
        "cost": [],
        "status": "active",
        "enabled": true,
        "limit": { "context": 100, "output": 20 },
    })
}

fn openai_model() -> Value {
    model(
        json!({ "type": "aisdk", "package": "@ai-sdk/openai", "url": "https://openai.example/v1" }),
        json!([]),
    )
}

fn session_with_variant(catalog: &Value, variant: &str) -> Value {
    json!({
        "id": "ses_model_variant",
        "projectID": "global",
        "title": "test",
        "model": { "id": catalog["id"], "providerID": catalog["providerID"], "variant": variant },
        "location": { "directory": "/project" },
    })
}

#[test]
fn maps_catalog_openai_ai_sdk_models_into_native_responses_routes() {
    let resolved = from_catalog_model(&openai_model(), None).expect("resolve");
    assert_eq!(resolved.id, "api-test-model");
    assert_eq!(resolved.provider, "test-provider");
    assert_eq!(resolved.route_id, "openai-responses");
    assert_eq!(resolved.base_url, "https://openai.example/v1");
    assert_eq!(resolved.headers, json!({ "x-test": "header" }));
    assert_eq!(resolved.limits, json!({ "context": 100, "output": 20 }));
    assert_eq!(
        resolved.body,
        json!({ "custom_extension": { "enabled": true } })
    );
}

#[test]
fn keeps_catalog_api_key_credentials_out_of_provider_json() {
    let resolved = from_catalog_model(&openai_model(), None).expect("resolve");
    let serialized = serde_json::to_string(&resolved.body).expect("resolve");
    assert!(!serialized.contains("apiKey"));
    assert!(!serialized.contains("secret"));
}

#[test]
fn uses_merged_api_settings_for_openai_compatible_auth_and_request_defaults() {
    let mut info = model(
        json!({
            "type": "aisdk",
            "package": "@ai-sdk/openai-compatible",
            "url": "https://compatible.example/v1",
            "settings": { "apiKey": "settings-secret", "compatibility": "strict" },
        }),
        json!([]),
    );
    info["request"] = json!({ "headers": {}, "body": {} });

    let resolved = from_catalog_model(&info, None).expect("resolve");
    assert_eq!(
        resolved.bearer(),
        Some("Bearer settings-secret".to_string())
    );
    assert_eq!(resolved.body, json!({}));
}

#[test]
fn overlays_selected_openai_session_variant_bodies() {
    let catalog = model(
        json!({ "type": "aisdk", "package": "@ai-sdk/openai", "url": "https://openai.example/v1" }),
        json!([{
            "id": "high",
            "headers": { "x-variant": "high" },
            "body": { "store": false, "service_tier": "priority", "temperature": 0.2, "reasoning": { "effort": "high" } },
        }]),
    );
    let resolved = resolve(&session_with_variant(&catalog, "high"), &catalog).expect("resolve");

    assert_eq!(
        resolved.headers,
        json!({ "x-test": "header", "x-variant": "high" })
    );
    assert_eq!(
        resolved.body,
        json!({
            "custom_extension": { "enabled": true },
            "store": false,
            "service_tier": "priority",
            "temperature": 0.2,
            "reasoning": { "effort": "high" },
        })
    );
}

#[test]
fn overlays_selected_openai_compatible_session_variant_bodies() {
    let catalog = model(
        json!({ "type": "aisdk", "package": "@ai-sdk/openai-compatible", "url": "https://compatible.example/v1" }),
        json!([{ "id": "high", "headers": {}, "body": { "store": false, "reasoning_effort": "high" } }]),
    );
    let resolved = resolve(&session_with_variant(&catalog, "high"), &catalog).expect("resolve");
    assert_eq!(
        resolved.body,
        json!({ "custom_extension": { "enabled": true }, "store": false, "reasoning_effort": "high" })
    );
}

#[test]
fn rejects_an_explicit_unavailable_session_variant_during_model_resolution() {
    let catalog = openai_model();
    let session = session_with_variant(&catalog, "unknown");
    let failure = resolve(&session, &catalog).expect_err("resolve");
    assert_eq!(
        failure.to_string(),
        variant_unavailable_message("test-provider", "test-model", "unknown")
    );
    assert_eq!(
        failure.to_string(),
        "Variant unavailable for test-provider/test-model: unknown"
    );
}

#[test]
fn overlays_selected_anthropic_session_variant_bodies() {
    let catalog = model(
        json!({ "type": "aisdk", "package": "@ai-sdk/anthropic", "url": "https://anthropic.example/v1" }),
        json!([{ "id": "high", "headers": {}, "body": { "thinking": { "type": "enabled", "budget_tokens": 12000 } } }]),
    );
    let resolved = resolve(&session_with_variant(&catalog, "high"), &catalog).expect("resolve");
    assert_eq!(
        resolved.body,
        json!({ "custom_extension": { "enabled": true }, "thinking": { "type": "enabled", "budget_tokens": 12000 } })
    );
}

#[test]
fn maps_catalog_anthropic_ai_sdk_models_into_native_routes() {
    let resolved = from_catalog_model(
        &model(
            json!({ "type": "aisdk", "package": "@ai-sdk/anthropic", "url": "https://anthropic.example/v1" }),
            json!([]),
        ),
        None,
    )
    .expect("resolve");
    assert_eq!(resolved.route_id, "anthropic-messages");
    assert_eq!(resolved.base_url, "https://anthropic.example/v1");
}

#[test]
fn uses_resolved_credentials_for_bearer_auth() {
    let mut info = openai_model();
    info["request"] = json!({ "headers": {}, "body": {} });
    let credential = json!({ "type": "key", "key": "secret" });
    let resolved = from_catalog_model(&info, Some(&credential)).expect("resolve");
    assert_eq!(resolved.bearer(), Some("Bearer secret".to_string()));
}

#[test]
fn prefers_stored_credentials_over_configured_auth() {
    let mut info = openai_model();
    info["request"] = json!({ "headers": {}, "body": { "apiKey": "configured-secret" } });
    let credential =
        json!({ "type": "key", "key": "stored-secret", "metadata": { "tenant": "work" } });
    let resolved = from_catalog_model(&info, Some(&credential)).expect("resolve");
    assert_eq!(resolved.bearer(), Some("Bearer stored-secret".to_string()));
    assert_eq!(resolved.body, json!({ "tenant": "work" }));
}

#[test]
fn does_not_project_oauth_account_metadata_into_the_request_body() {
    let mut info = openai_model();
    info["request"] = json!({ "headers": {}, "body": {} });
    let credential = json!({
        "type": "oauth",
        "methodID": "device",
        "access": "secret",
        "refresh": "refresh",
        "metadata": { "server": "https://console.example", "orgID": "org_123" },
    });
    let resolved = from_catalog_model(&info, Some(&credential)).expect("resolve");
    assert_eq!(resolved.body, json!({}));
}

#[test]
fn rejects_catalog_apis_without_a_native_route() {
    let failure = from_catalog_model(
        &model(
            json!({ "type": "aisdk", "package": "@ai-sdk/google", "url": "https://google.example/v1" }),
            json!([]),
        ),
        None,
    )
    .expect_err("resolve");
    assert_eq!(
        failure.to_string(),
        unsupported_api_message("test-provider", "test-model", "aisdk:@ai-sdk/google"),
        "Unsupported API for test-provider/test-model: aisdk:@ai-sdk/google"
    );
}

#[test]
fn reports_whether_a_catalog_model_has_a_supported_native_route() {
    assert!(supported(&openai_model()));
    assert!(!supported(&model(
        json!({ "type": "aisdk", "package": "@ai-sdk/google", "url": "https://google.example/v1" }),
        json!([]),
    )));
    assert!(!supported(&model(
        json!({ "type": "native", "settings": {} }),
        json!([])
    )));
}
