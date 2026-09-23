//! Port of packages/core/test/config/provider.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: configured provider documents merge in order, later model
//! overrides win, request headers/body merge with the last document resolving
//! `shared`, configured models keep their variant bodies unchanged, and the last
//! `model` declaration selects the default. Re-derived: the Effect service wiring
//! and env acquisition are replaced by a plain document merge over JSON.

use opencode_core::provider::ConfigProviderPlugin;
use serde_json::json;

fn documents() -> Vec<serde_json::Value> {
    vec![
        json!({
            "model": "custom/first",
            "providers": {
                "custom": {
                    "name": "Configured",
                    "env": ["CUSTOM_API_KEY"],
                    "api": { "type": "native", "settings": {} },
                    "request": { "headers": { "first": "first", "shared": "first" } },
                    "models": {
                        "chat": {
                            "name": "First",
                            "capabilities": { "tools": true, "input": ["text"], "output": ["text"] },
                            "disabled": true,
                            "limit": { "context": 100, "output": 50 },
                            "cost": { "input": 1, "output": 2 },
                            "request": { "headers": { "first": "first", "shared": "first" }, "variant": "retained" },
                            "variants": [
                                { "id": "fast", "headers": { "first": "first", "shared": "first" } }
                            ]
                        }
                    }
                }
            }
        }),
        json!({
            "model": "custom/default",
            "providers": {
                "custom": {
                    "api": { "type": "aisdk", "package": "custom-sdk", "url": "https://example.test" },
                    "request": { "headers": { "last": "last", "shared": "last" } },
                    "models": {
                        "default": { "name": "Default" },
                        "chat": {
                            "api": { "id": "api-chat" },
                            "name": "Last",
                            "limit": { "output": 75 },
                            "request": { "headers": { "last": "last", "shared": "last" } },
                            "variants": [
                                { "id": "fast", "headers": { "last": "last", "shared": "last" } },
                                { "id": "slow", "headers": { "slow": "slow" } }
                            ]
                        }
                    }
                }
            }
        }),
        json!({ "providers": { "custom": { "name": "Renamed" } } }),
    ]
}

#[test]
fn loads_configured_providers_and_applies_later_model_overrides() {
    let snapshot = ConfigProviderPlugin::build(&documents()).unwrap();

    assert_eq!(snapshot.default_model.as_deref(), Some("custom/default"));

    let provider = snapshot.providers.get("custom").unwrap();
    assert_eq!(provider.name, "Renamed");
    assert!(!provider.disabled);
    assert_eq!(
        provider.api,
        json!({ "type": "aisdk", "package": "custom-sdk", "url": "https://example.test" })
    );
    assert_eq!(
        provider.request["headers"],
        json!({ "first": "first", "shared": "last", "last": "last" })
    );

    let model = provider.models.get("chat").unwrap();
    assert_eq!(model.api["id"], json!("api-chat"));
    assert_eq!(model.name, "Last");
    assert_eq!(model.limit, json!({ "context": 100, "output": 75 }));
    assert!(!model.enabled);
    assert_eq!(model.cost, json!([{ "input": 1, "output": 2 }]));
    assert_eq!(model.request["variant"], json!("retained"));
    assert_eq!(
        model.request["headers"],
        json!({ "first": "first", "shared": "last", "last": "last" })
    );
    let ids: Vec<&str> = model
        .variants
        .iter()
        .map(|variant| variant.id.as_str())
        .collect();
    assert_eq!(ids, vec!["fast", "slow"]);
    assert_eq!(
        model.variants[0].headers,
        json!({ "first": "first", "shared": "last", "last": "last" })
    );
    assert_eq!(model.variants[1].headers, json!({ "slow": "slow" }));
}

#[test]
fn keeps_configured_model_variant_bodies_unchanged() {
    let docs = vec![json!({
        "providers": {
            "opencode": {
                "api": { "type": "aisdk", "package": "@ai-sdk/openai", "url": "https://opencode.test/v1" },
                "models": {
                    "alpha-gpt-next": {
                        "variants": [{
                            "id": "high",
                            "body": {
                                "reasoningEffort": "high",
                                "reasoningSummary": "auto",
                                "include": ["reasoning.encrypted_content"]
                            }
                        }]
                    }
                }
            }
        }
    })];

    let snapshot = ConfigProviderPlugin::build(&docs).unwrap();
    let model = snapshot
        .providers
        .get("opencode")
        .and_then(|provider| provider.models.get("alpha-gpt-next"))
        .unwrap();
    assert_eq!(
        model.variants[0].body,
        json!({
            "reasoningEffort": "high",
            "reasoningSummary": "auto",
            "include": ["reasoning.encrypted_content"]
        })
    );
}
