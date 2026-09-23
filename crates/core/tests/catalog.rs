//! Port of packages/core/test/catalog.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: provider/model `baseURL` normalization into the API url,
//! provider/model request-map merging, and `default`/`small` selection. Dropped
//! (re-derived): the availability tests and the event/credential/policy wiring
//! cases, which depend on Effect streams, the database, and service layers.

use opencode_core::catalog::Catalog;
use serde_json::{json, Value};

fn required(value: Option<Value>) -> Value {
    value.expect("expected value")
}

#[test]
fn normalizes_provider_base_url_into_api_url() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |provider| {
                provider["api"] = json!({
                    "type": "aisdk",
                    "package": "@ai-sdk/openai-compatible",
                    "url": "https://default.example.com",
                });
                provider["request"]["body"]["baseURL"] = json!("https://override.example.com");
            });
        })
        .unwrap();

    let provider = required(catalog.provider_get("test").unwrap());
    assert_eq!(
        provider["api"],
        json!({
            "type": "aisdk",
            "package": "@ai-sdk/openai-compatible",
            "url": "https://override.example.com",
        })
    );
}

#[test]
fn normalizes_model_base_url_into_api_url() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |provider| {
                provider["api"] = json!({
                    "type": "aisdk",
                    "package": "@ai-sdk/openai-compatible",
                    "url": "https://provider.example.com",
                });
            });
            catalog.model_update("test", "model", |model| {
                model["api"] = json!({
                    "id": "model",
                    "type": "aisdk",
                    "package": "@ai-sdk/openai-compatible",
                    "url": "https://model.example.com",
                });
                model["request"]["body"]["baseURL"] = json!("https://override.example.com");
            });
        })
        .unwrap();

    let model = required(catalog.model_get("test", "model").unwrap());
    assert_eq!(
        model["api"],
        json!({
            "id": "model",
            "type": "aisdk",
            "package": "@ai-sdk/openai-compatible",
            "url": "https://override.example.com",
            "settings": {},
        })
    );
}

#[test]
fn resolves_default_model_api_from_provider_api() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |provider| {
                provider["api"] = json!({
                    "type": "aisdk",
                    "package": "@ai-sdk/openai-compatible",
                    "url": "https://provider.example.com",
                });
            });
            catalog.model_update("test", "model", |_| {});
        })
        .unwrap();

    let model = required(catalog.model_get("test", "model").unwrap());
    assert_eq!(
        model["api"],
        json!({
            "id": "model",
            "type": "aisdk",
            "package": "@ai-sdk/openai-compatible",
            "url": "https://provider.example.com",
        })
    );
}

#[test]
fn resolves_provider_and_model_request_merges() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |provider| {
                provider["request"]["headers"]["provider"] = json!("provider");
                provider["request"]["headers"]["shared"] = json!("provider");
                provider["request"]["body"]["provider"] = json!(true);
            });
            catalog.model_update("test", "model", |model| {
                model["request"]["headers"]["model"] = json!("model");
                model["request"]["headers"]["shared"] = json!("model");
                model["request"]["body"]["model"] = json!(true);
                model["request"]["body"]["request"] = json!(true);
                model["request"]["body"]["shared"] = json!("model");
            });
        })
        .unwrap();

    let model = required(catalog.model_get("test", "model").unwrap());
    assert_eq!(
        model["request"]["headers"],
        json!({ "provider": "provider", "shared": "model", "model": "model" })
    );
    assert_eq!(
        model["request"]["body"],
        json!({ "provider": true, "model": true, "request": true, "shared": "model" })
    );
}

#[test]
fn falls_back_to_newest_available_model_when_no_default_is_configured() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |_| {});
            catalog.model_update("test", "old", |model| {
                model["time"]["released"] = json!(1000);
            });
            catalog.model_update("test", "new", |model| {
                model["time"]["released"] = json!(2000);
            });
        })
        .unwrap();

    let default = required(catalog.model_default().unwrap());
    assert_eq!(default["id"], json!("new"));
}

#[test]
fn uses_transform_provided_default_until_that_transform_is_replaced() {
    use std::cell::Cell;
    use std::rc::Rc;

    let catalog = Catalog::new().unwrap();
    let configured = Rc::new(Cell::new(true));
    let captured = Rc::clone(&configured);
    catalog
        .transform(move |catalog| {
            catalog.provider_update("test", |_| {});
            catalog.model_update("test", "old", |model| {
                model["time"]["released"] = json!(1000);
            });
            catalog.model_update("test", "new", |model| {
                model["time"]["released"] = json!(2000);
            });
            if captured.get() {
                catalog.default_set("test", "old");
            }
        })
        .unwrap();
    assert_eq!(
        required(catalog.model_default().unwrap())["id"],
        json!("old")
    );

    configured.set(false);
    catalog.reload().unwrap();
    assert_eq!(
        required(catalog.model_default().unwrap())["id"],
        json!("new")
    );
}

#[test]
fn ignores_a_configured_default_on_a_disabled_provider() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("disabled", |provider| {
                provider["disabled"] = json!(true);
            });
            catalog.model_update("disabled", "configured", |_| {});
            catalog.provider_update("enabled", |_| {});
            catalog.model_update("enabled", "fallback", |_| {});
            catalog.default_set("disabled", "configured");
        })
        .unwrap();

    let default = required(catalog.model_default().unwrap());
    assert_eq!(default["providerID"], json!("enabled"));
    assert_eq!(default["id"], json!("fallback"));
}

#[test]
fn small_model_prefers_small_keyword_candidates_before_cost_scoring() {
    let catalog = Catalog::new().unwrap();
    catalog
        .transform(|catalog| {
            catalog.provider_update("test", |_| {});
            catalog.model_update("test", "cheap-large", |model| {
                model["capabilities"]["input"] = json!(["text"]);
                model["capabilities"]["output"] = json!(["text"]);
                model["cost"] =
                    json!([{ "input": 1, "output": 1, "cache": { "read": 0, "write": 0 } }]);
                model["time"]["released"] = json!(1_700_000_000_000_i64);
            });
            catalog.model_update("test", "expensive-mini", |model| {
                model["capabilities"]["input"] = json!(["text"]);
                model["capabilities"]["output"] = json!(["text"]);
                model["cost"] =
                    json!([{ "input": 10, "output": 10, "cache": { "read": 0, "write": 0 } }]);
                model["time"]["released"] = json!(1_700_000_000_000_i64);
            });
        })
        .unwrap();

    let small = required(catalog.model_small("test").unwrap());
    assert_eq!(small["id"], json!("expensive-mini"));
}
