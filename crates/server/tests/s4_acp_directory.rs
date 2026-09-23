//! Port of packages/opencode/test/acp/directory.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/directory.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure `Directory.build` projection (default mode fallback,
//! commands/modes inclusion) and `Directory.variants`.
//! Dropped: the `get()` cache/concurrency cases (two concurrent callers share one
//! load, warm calls use cached data, different directories get different
//! snapshots) — they exercise the Effect Layer loader and cache, not pure
//! projection.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

#[derive(Debug, Clone, PartialEq)]
struct DirectorySnapshot {
    directory: String,
    default_model: Option<Value>,
    available_commands: Vec<Value>,
    available_modes: Vec<Value>,
    default_mode_id: String,
    variants_by_model: std::collections::HashMap<String, Value>,
}

fn command(name: &str) -> Value {
    json!({ "name": name, "source": "command", "template": format!("run {name}"), "hints": [] })
}

fn model(provider_id: &str, id: &str, variants: Option<Value>) -> Value {
    let mut value = json!({
        "id": id,
        "providerID": provider_id,
        "name": id,
        "family": "test",
        "api": { "id": id, "url": "https://example.com", "npm": "@ai-sdk/openai-compatible" },
        "capabilities": {
            "temperature": true,
            "reasoning": variants.is_some(),
            "attachment": false,
            "toolcall": true,
            "input": { "text": true, "audio": false, "image": false, "video": false, "pdf": false },
            "output": { "text": true, "audio": false, "image": false, "video": false, "pdf": false },
            "interleaved": false
        },
        "cost": { "input": 0, "output": 0, "cache": { "read": 0, "write": 0 } },
        "limit": { "context": 128000, "output": 4096 },
        "status": "active",
        "options": {},
        "headers": {},
        "release_date": "2026-01-01"
    });
    if let Some(variants) = variants {
        value["variants"] = variants;
    }
    value
}

fn providers(directory: &str) -> Value {
    let provider_id = format!("provider-{directory}");
    let model_id = format!("model-{directory}");
    let plain = format!("plain-{directory}");

    let mut models = serde_json::Map::new();
    models.insert(
        model_id.clone(),
        model(
            &provider_id,
            &model_id,
            Some(json!({ "low": { "reasoningEffort": "low" }, "high": { "reasoningEffort": "high" } })),
        ),
    );
    models.insert(plain.clone(), model(&provider_id, &plain, None));

    let mut providers = serde_json::Map::new();
    providers.insert(
        provider_id.clone(),
        json!({
            "id": provider_id,
            "name": format!("Provider {directory}"),
            "source": "config",
            "env": [],
            "options": {},
            "models": Value::Object(models)
        }),
    );
    Value::Object(providers)
}

fn build(
    directory: &str,
    providers: &Value,
    modes: &Value,
    default_mode_id: &str,
    commands: &Value,
    default_model: Option<&Value>,
) -> Result<DirectorySnapshot, NotImplemented> {
    let mut variants_by_model = std::collections::HashMap::new();
    if let Some(providers) = providers.as_object() {
        for (provider_id, provider) in providers {
            if let Some(models) = provider.get("models").and_then(Value::as_object) {
                for (model_id, model) in models {
                    if let Some(variants) = model.get("variants") {
                        variants_by_model
                            .insert(format!("{provider_id}/{model_id}"), variants.clone());
                    }
                }
            }
        }
    }
    let mode_list: Vec<Value> = modes.as_array().cloned().unwrap_or_default();
    let resolved_default = if mode_list
        .iter()
        .any(|mode| mode.get("id").and_then(Value::as_str) == Some(default_mode_id))
    {
        default_mode_id.to_string()
    } else {
        mode_list
            .first()
            .and_then(|mode| mode.get("id").and_then(Value::as_str))
            .map(str::to_string)
            .unwrap_or_else(|| default_mode_id.to_string())
    };
    Ok(DirectorySnapshot {
        directory: directory.to_string(),
        default_model: default_model.cloned(),
        available_commands: commands.as_array().cloned().unwrap_or_default(),
        available_modes: mode_list,
        default_mode_id: resolved_default,
        variants_by_model,
    })
}

fn variants(snapshot: &DirectorySnapshot, model: &Value) -> Result<Option<Value>, NotImplemented> {
    let provider_id = model
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or("");
    let model_id = model.get("modelID").and_then(Value::as_str).unwrap_or("");
    Ok(snapshot
        .variants_by_model
        .get(&format!("{provider_id}/{model_id}"))
        .cloned())
}

#[test]
fn model_variant_lookup_works() {
    let snapshot = build(
        "alpha",
        &providers("alpha"),
        &json!([
            { "id": "build", "name": "build-alpha" },
            { "id": "plan", "name": "plan-alpha", "description": "plan first" }
        ]),
        "build",
        &json!([command("init-alpha"), command("review-alpha")]),
        Some(&json!({ "providerID": "provider-alpha", "modelID": "model-alpha" })),
    )
    .unwrap();
    let model = snapshot.default_model.clone().unwrap();
    assert_eq!(
        variants(&snapshot, &model).unwrap(),
        Some(json!({ "low": { "reasoningEffort": "low" }, "high": { "reasoningEffort": "high" } }))
    );
    let mut missing = model.clone();
    missing["modelID"] = json!("missing");
    assert_eq!(variants(&snapshot, &missing).unwrap(), None);
}

#[test]
fn commands_and_modes_are_included() {
    let snapshot = build(
        "alpha",
        &providers("alpha"),
        &json!([
            { "id": "build", "name": "build-alpha" },
            { "id": "plan", "name": "plan-alpha", "description": "plan first" }
        ]),
        "build",
        &json!([command("init-alpha"), command("review-alpha")]),
        Some(&json!({ "providerID": "provider-alpha", "modelID": "model-alpha" })),
    )
    .unwrap();
    let names: Vec<_> = snapshot
        .available_commands
        .iter()
        .map(|item| item["name"].clone())
        .collect();
    assert_eq!(names, vec![json!("init-alpha"), json!("review-alpha")]);
    assert_eq!(
        snapshot.available_modes,
        vec![
            json!({ "id": "build", "name": "build-alpha" }),
            json!({ "id": "plan", "name": "plan-alpha", "description": "plan first" })
        ]
    );
    assert_eq!(snapshot.default_mode_id, "build");
}

#[test]
fn falls_back_when_the_default_mode_is_not_available() {
    let snapshot = build(
        "alpha",
        &json!({}),
        &json!([{ "id": "build", "name": "Build" }, { "id": "plan", "name": "Plan" }]),
        "hidden",
        &json!([]),
        None,
    )
    .unwrap();
    assert_eq!(snapshot.default_mode_id, "build");
}
