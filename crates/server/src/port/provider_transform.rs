//! Provider schema transforms.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/opencode/test/provider/transform.test.ts` (upstream 18ef3cc):
//! `ProviderTransform.schema` for Gemini (array items, type arrays, combiner
//! nodes, non-object cleanup), OpenAI (supported-keyword subset), and Moonshot
//! (`$ref` siblings, tuple items).
//!
//! Dropped (needs the live SDK/catalog): `ProviderTransform.options`,
//! `providerOptions`, and `message` cases.

use serde_json::{json, Map, Value};

/// Apply the provider-specific JSON Schema transform for a model.
///
/// `provider_id`, `api_id`, and `npm` select the transform exactly as the
/// reference does: Moonshot (`kimi`) first, then Gemini, then the OpenAI SDK
/// subset. Unknown models are returned unchanged.
pub fn schema(provider_id: &str, api_id: &str, npm: &str, schema: &Value) -> Value {
    if is_moonshot(provider_id, api_id) {
        let mut out = schema.clone();
        sanitize_moonshot(&mut out);
        out
    } else if is_gemini(provider_id, api_id) {
        let mut out = schema.clone();
        sanitize_gemini(&mut out);
        out
    } else if is_openai(npm) {
        sanitize_openai(schema)
    } else {
        schema.clone()
    }
}

fn is_gemini(provider_id: &str, api_id: &str) -> bool {
    provider_id == "google" || provider_id == "google-vertex" || api_id.contains("gemini")
}

fn is_moonshot(provider_id: &str, api_id: &str) -> bool {
    provider_id == "moonshotai" || api_id.contains("kimi")
}

fn is_openai(npm: &str) -> bool {
    npm == "@ai-sdk/openai" || npm == "@ai-sdk/azure" || npm.starts_with("@ai-sdk/openai")
}

fn recurse_mut(node: &mut Value, f: fn(&mut Value)) {
    let Some(object) = node.as_object_mut() else {
        return;
    };
    for key in ["properties", "$defs"] {
        if let Some(Value::Object(map)) = object.get_mut(key) {
            for value in map.values_mut() {
                f(value);
            }
        }
    }
    if let Some(items) = object.get_mut("items") {
        match items {
            Value::Array(list) => {
                for value in list {
                    f(value);
                }
            }
            other => f(other),
        }
    }
    for key in ["anyOf", "oneOf", "allOf"] {
        if let Some(Value::Array(list)) = object.get_mut(key) {
            for value in list {
                f(value);
            }
        }
    }
    if let Some(additional) = object.get_mut("additionalProperties") {
        if additional.is_object() {
            f(additional);
        }
    }
}

fn sanitize_gemini(node: &mut Value) {
    if !node.is_object() {
        return;
    }
    {
        let object = node.as_object_mut().expect("checked object");
        if let Some(Value::Array(types)) = object.get("type").cloned() {
            let mut non_null: Vec<Value> = Vec::new();
            let mut has_null = false;
            for kind in &types {
                if kind.as_str() == Some("null") {
                    has_null = true;
                } else {
                    non_null.push(kind.clone());
                }
            }
            if non_null.is_empty() {
                object.insert("type".to_string(), json!("null"));
                object.remove("anyOf");
            } else if non_null.len() == 1 && !has_null {
                object.insert("type".to_string(), non_null[0].clone());
            } else {
                object.remove("type");
                if has_null {
                    object.insert("nullable".to_string(), json!(true));
                }
                object.insert(
                    "anyOf".to_string(),
                    Value::Array(
                        non_null
                            .into_iter()
                            .map(|kind| json!({ "type": kind }))
                            .collect(),
                    ),
                );
            }
        }

        if object.get("type").and_then(Value::as_str) == Some("array")
            && !object.contains_key("items")
        {
            object.insert("items".to_string(), json!({ "type": "string" }));
        }

        if let Some(item) = object.get_mut("items").and_then(Value::as_object_mut) {
            let has_type = item.contains_key("type");
            let has_combiner = item.contains_key("anyOf")
                || item.contains_key("oneOf")
                || item.contains_key("allOf");
            let has_ref = item.contains_key("$ref");
            let has_properties = item.contains_key("properties");
            if !has_type && !has_combiner && !has_ref && !has_properties {
                item.insert("type".to_string(), json!("string"));
            }
        }

        if object
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind != "object")
        {
            object.remove("properties");
            object.remove("required");
        }
    }
    recurse_mut(node, sanitize_gemini);
}

fn sanitize_moonshot(node: &mut Value) {
    if !node.is_object() {
        return;
    }
    {
        let object = node.as_object_mut().expect("checked object");
        if object.contains_key("$ref") {
            let reference = object.get("$ref").cloned().unwrap_or(Value::Null);
            object.clear();
            object.insert("$ref".to_string(), reference);
            return;
        }
        if object.get("type").and_then(Value::as_str) == Some("array") {
            let first = match object.get("items") {
                Some(Value::Array(list)) => list.first().cloned(),
                _ => None,
            };
            if let Some(first) = first {
                object.insert("items".to_string(), first);
            }
        }
    }
    recurse_mut(node, sanitize_moonshot);
}

fn sanitize_openai(node: &Value) -> Value {
    let Some(object) = node.as_object() else {
        return node.clone();
    };
    if object.contains_key("$ref") {
        let mut out = Map::new();
        out.insert(
            "$ref".to_string(),
            object.get("$ref").cloned().unwrap_or(Value::Null),
        );
        if let Some(description) = object.get("description") {
            out.insert("description".to_string(), description.clone());
        }
        return Value::Object(out);
    }

    let has_combiner = object.contains_key("anyOf")
        || object.contains_key("oneOf")
        || object.contains_key("allOf");
    let mut out = Map::new();

    let mut node_type = object.get("type").cloned();
    if let Some(constant) = object.get("const") {
        out.insert("enum".to_string(), json!([constant.clone()]));
        node_type = Some(json!("string"));
    }
    if let Some(enumeration) = object.get("enum") {
        out.insert("enum".to_string(), enumeration.clone());
    }
    if node_type.is_none() && !has_combiner {
        node_type = Some(json!("string"));
    }
    if let Some(kind) = &node_type {
        out.insert("type".to_string(), kind.clone());
    }
    if let Some(description) = object.get("description") {
        out.insert("description".to_string(), description.clone());
    }

    if let Some(properties) = object.get("properties").and_then(Value::as_object) {
        let mut projected = Map::new();
        for (key, value) in properties {
            projected.insert(key.clone(), sanitize_openai(value));
        }
        out.insert("properties".to_string(), Value::Object(projected));
    } else if node_type.as_ref().and_then(Value::as_str) == Some("object") {
        out.insert("properties".to_string(), json!({}));
    }

    if let Some(required) = object.get("required") {
        out.insert("required".to_string(), required.clone());
    }

    if let Some(items) = object.get("items") {
        let projected = match items {
            Value::Array(list) => Value::Array(list.iter().map(sanitize_openai).collect()),
            other => sanitize_openai(other),
        };
        out.insert("items".to_string(), projected);
    } else if node_type.as_ref().and_then(Value::as_str) == Some("array") {
        out.insert("items".to_string(), json!({ "type": "string" }));
    }

    if let Some(additional) = object.get("additionalProperties") {
        if additional.is_boolean() {
            out.insert("additionalProperties".to_string(), additional.clone());
        } else if additional.is_object() {
            out.insert(
                "additionalProperties".to_string(),
                sanitize_openai(additional),
            );
        }
    }

    if let Some(defs) = object.get("$defs").and_then(Value::as_object) {
        let mut projected = Map::new();
        for (key, value) in defs {
            projected.insert(key.clone(), sanitize_openai(value));
        }
        out.insert("$defs".to_string(), Value::Object(projected));
    }

    for key in ["anyOf", "oneOf", "allOf"] {
        if let Some(Value::Array(list)) = object.get(key) {
            out.insert(
                key.to_string(),
                Value::Array(list.iter().map(sanitize_openai).collect()),
            );
        }
    }

    Value::Object(out)
}
