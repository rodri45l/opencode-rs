//! Port of packages/core/test/plugin/models-dev.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: experimental modes are projected as separate models instead
//! of variants, their cost list is normalized (mode cost, context tiers, then the
//! legacy `context_over_200k` tier), and providers with environment variables
//! register a key method plus an env method. Re-derived: the `Catalog`/
//! `Integration`/`ModelsDev` service wiring is replaced by a direct projection.

use std::collections::BTreeMap;

use opencode_core::models_dev_plugin::{ModelsDevModel, ModelsDevPlugin, ModelsDevProvider};
use serde_json::json;

fn provider() -> ModelsDevProvider {
    let mut modes = BTreeMap::new();
    modes.insert(
        "fast".to_string(),
        json!({
            "cost": { "input": 5, "output": 30, "cache_read": 0.5 },
            "provider": {
                "headers": { "x-mode": "fast" },
                "body": { "service_tier": "priority" }
            }
        }),
    );
    let mut models = BTreeMap::new();
    models.insert(
        "gpt-5.4".to_string(),
        ModelsDevModel {
            id: "gpt-5.4".into(),
            name: "GPT-5.4".into(),
            cost: json!({
                "input": 2.5,
                "output": 15,
                "tiers": [{ "tier": { "type": "context", "size": 272_000 }, "input": 3, "output": 18, "cache_read": 0.25 }],
                "context_over_200k": { "input": 5, "output": 22.5, "cache_read": 0.5 }
            }),
            limit: json!({ "context": 1_050_000, "input": 922_000, "output": 128_000 }),
            modes,
        },
    );
    ModelsDevProvider {
        id: "acme".into(),
        name: "Acme".into(),
        env: vec!["ACME_API_KEY".into()],
        npm: Some("@ai-sdk/openai-compatible".into()),
        api: Some("https://api.acme.test/v1".into()),
        models,
    }
}

#[test]
fn projects_modes_as_separate_models_instead_of_variants() {
    let projected = ModelsDevPlugin::project_models(&provider()).unwrap();

    let base = projected
        .iter()
        .find(|model| model.id == "gpt-5.4")
        .unwrap();
    assert!(base.variants.is_empty());
    assert_eq!(base.request["body"], json!({}));

    let fast = projected
        .iter()
        .find(|model| model.id == "gpt-5.4-fast")
        .unwrap();
    assert_eq!(fast.name, "GPT-5.4 Fast");
    assert_eq!(fast.api_id, "gpt-5.4");
    assert_eq!(fast.request["headers"], json!({ "x-mode": "fast" }));
    assert_eq!(fast.request["body"], json!({ "service_tier": "priority" }));
    assert!(fast.variants.is_empty());
    assert_eq!(
        fast.cost,
        vec![
            json!({ "input": 5, "output": 30, "cache": { "read": 0.5, "write": 0 } }),
            json!({ "tier": { "type": "context", "size": 272_000 }, "input": 3, "output": 18, "cache": { "read": 0.25, "write": 0 } }),
            json!({ "tier": { "type": "context", "size": 200_000 }, "input": 5, "output": 22.5, "cache": { "read": 0.5, "write": 0 } })
        ]
    );
}

#[test]
fn registers_key_methods_for_providers_with_environment_variables() {
    let methods = ModelsDevPlugin::integration_methods(&provider()).unwrap();
    assert_eq!(
        methods,
        vec![
            json!({ "type": "key" }),
            json!({ "type": "env", "names": ["ACME_API_KEY"] })
        ]
    );
}
