//! Port of packages/opencode/test/provider/digitalocean.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/provider/provider.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure DigitalOcean router model projection — `router:<name>` ids
//! that share the base OpenAI-compatible endpoint, and that base (non-router)
//! models are the only ones surfaced when no auth metadata is present.
//! Dropped: the `it.instance` provider-loading cases (autoload from
//! `DIGITALOCEAN_ACCESS_TOKEN`, cached routers from auth metadata, skipping
//! refresh when the OAuth bearer is expired). Those need config/env/auth and the
//! live provider service.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn router_model(_name: &str) -> Result<Value, NotImplemented> {
    nope("digitalocean")
}

fn base_models_only(_models: &Value) -> Result<Vec<String>, NotImplemented> {
    nope("digitalocean")
}

#[test]
#[ignore = "porting: digitalocean not implemented"]
fn router_models_use_router_prefixed_ids_and_the_base_endpoint() {
    assert_eq!(
        router_model("my-router").unwrap(),
        json!({
            "id": "router:my-router",
            "api": {
                "id": "router:my-router",
                "url": "https://inference.do-ai.run/v1",
                "npm": "@ai-sdk/openai-compatible"
            }
        })
    );
}

#[test]
#[ignore = "porting: digitalocean not implemented"]
fn base_models_are_passed_through_without_router_entries() {
    let models = json!({ "llama-3.3-70b": {}, "router:stale": {} });
    assert_eq!(
        base_models_only(&models).unwrap(),
        vec!["llama-3.3-70b".to_string()]
    );
}
