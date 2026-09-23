//! Protocol-specific tool schema projections.

use serde_json::{json, Map, Value};

use crate::error::LlmResult;

/// Provider-specific JSON Schema projections.
pub struct ToolSchemaProjection;

impl ToolSchemaProjection {
    /// Moonshot: strip `$ref` siblings and flatten tuple arrays.
    pub fn moonshot(schema: Value) -> LlmResult<Value> {
        Ok(moonshot(schema))
    }

    /// Gemini: normalise numeric enums, dangling required, untyped arrays.
    pub fn gemini(schema: Value) -> LlmResult<Value> {
        Ok(gemini(schema))
    }

    /// OpenAI: flatten top-level object unions into one object schema.
    pub fn open_ai(schema: Value) -> LlmResult<Value> {
        Ok(open_ai(schema))
    }

    /// Apply the model's declared tool-schema compatibility before protocol projection.
    pub fn model_compatibility(schema: Value, compatibility: Option<&str>) -> Value {
        match compatibility {
            Some("moonshot") => moonshot(schema),
            Some("gemini") => gemini(schema),
            _ => schema,
        }
    }
}

fn recurse(value: &Value, f: &dyn Fn(Value) -> Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(|item| recurse(item, f)).collect()),
        Value::Object(map) => {
            let mut out = Map::new();
            for (key, item) in map {
                out.insert(key.clone(), recurse(item, f));
            }
            f(Value::Object(out))
        }
        other => other.clone(),
    }
}

fn moonshot(schema: Value) -> Value {
    recurse(&schema, &|mut value| {
        let Some(map) = value.as_object_mut() else {
            return value;
        };
        if map.contains_key("$ref") {
            let reference = map.get("$ref").cloned().unwrap();
            let mut out = Map::new();
            out.insert("$ref".into(), reference);
            return Value::Object(out);
        }
        if let Some(items) = map.get("items").cloned() {
            if items.is_array() {
                map.insert("items".into(), json!({ "anyOf": items }));
            }
        }
        if let Some(prefix) = map.remove("prefixItems") {
            if map.get("items").is_none() {
                map.insert("items".into(), json!({ "anyOf": prefix }));
            }
        }
        value
    })
}

fn stringify_enum(value: &Value) -> Value {
    match value {
        Value::Number(n) => Value::String(n.to_string()),
        Value::Bool(b) => Value::String(b.to_string()),
        other => other.clone(),
    }
}

fn gemini(schema: Value) -> Value {
    fn walk(value: &Value) -> Value {
        match value {
            Value::Array(items) => Value::Array(items.iter().map(walk).collect()),
            Value::Object(map) => {
                let mut out = Map::new();
                let schema_type = map.get("type").and_then(Value::as_str).map(str::to_string);

                for (key, item) in map {
                    match key.as_str() {
                        "required" => {
                            if schema_type.as_deref() != Some("object") {
                                // Drop `required` on non-object scalars.
                                continue;
                            }
                            // Drop `required` entries with no matching property.
                            if let Some(properties) =
                                map.get("properties").and_then(Value::as_object)
                            {
                                if let Some(list) = item.as_array() {
                                    let filtered: Vec<Value> = list
                                        .iter()
                                        .filter(|entry| {
                                            entry
                                                .as_str()
                                                .map(|name| properties.contains_key(name))
                                                .unwrap_or(false)
                                        })
                                        .cloned()
                                        .collect();
                                    out.insert("required".into(), Value::Array(filtered));
                                    continue;
                                }
                            }
                            out.insert(key.clone(), item.clone());
                        }
                        "properties" if schema_type.as_deref() != Some("object") => {
                            // Drop properties on non-object scalars.
                        }
                        "properties" => {
                            if let Some(props) = item.as_object() {
                                let mut projected = Map::new();
                                for (name, prop) in props {
                                    projected.insert(name.clone(), walk(prop));
                                }
                                out.insert("properties".into(), Value::Object(projected));
                            }
                        }
                        "enum" => {
                            let values = item
                                .as_array()
                                .map(|list| Value::Array(list.iter().map(stringify_enum).collect()))
                                .unwrap_or_else(|| item.clone());
                            out.insert("enum".into(), values);
                            if schema_type.as_deref() == Some("integer")
                                || schema_type.as_deref() == Some("number")
                            {
                                out.insert("type".into(), Value::String("string".into()));
                            }
                        }
                        "type" => {
                            if !out.contains_key("type") {
                                out.insert(key.clone(), item.clone());
                            }
                        }
                        "items" => {
                            out.insert("items".into(), walk(item));
                        }
                        _ => {
                            out.insert(key.clone(), walk(item));
                        }
                    }
                }

                if schema_type.as_deref() == Some("array") && !out.contains_key("items") {
                    out.insert("items".into(), json!({ "type": "string" }));
                }
                Value::Object(out)
            }
            other => other.clone(),
        }
    }
    walk(&schema)
}

fn collapse_nullable(value: &Value) -> Value {
    let Some(map) = value.as_object() else {
        return value.clone();
    };
    for key in ["anyOf", "oneOf"] {
        if let Some(branches) = map.get(key).and_then(Value::as_array) {
            let non_null: Vec<&Value> = branches
                .iter()
                .filter(|branch| branch.get("type").and_then(Value::as_str) != Some("null"))
                .collect();
            if non_null.len() == 1 && non_null.len() != branches.len() {
                return non_null[0].clone();
            }
        }
    }
    value.clone()
}

fn open_ai(schema: Value) -> Value {
    fn walk(value: &Value) -> Value {
        let value = collapse_nullable(value);
        match value {
            Value::Array(items) => Value::Array(items.iter().map(walk).collect()),
            Value::Object(map) => {
                let branches = map
                    .get("anyOf")
                    .or_else(|| map.get("oneOf"))
                    .and_then(Value::as_array)
                    .cloned();
                let union_of_objects = branches.as_ref().map(|branches| {
                    !branches.is_empty()
                        && branches.iter().all(|branch| {
                            branch.get("properties").is_some()
                                || branch.get("type").and_then(Value::as_str) == Some("object")
                        })
                });
                if union_of_objects == Some(true) && !map.contains_key("properties") {
                    let mut out = Map::new();
                    out.insert("type".into(), Value::String("object".into()));
                    let mut properties = Map::new();
                    for branch in branches.as_ref().unwrap() {
                        if let Some(props) = branch.get("properties").and_then(Value::as_object) {
                            for (name, prop) in props {
                                properties.entry(name.clone()).or_insert_with(|| walk(prop));
                            }
                        }
                    }
                    if !properties.is_empty() {
                        out.insert("properties".into(), Value::Object(properties));
                    }
                    out.insert("additionalProperties".into(), Value::Bool(false));
                    if let Some(description) = map.get("description") {
                        out.insert("description".into(), description.clone());
                    }
                    return Value::Object(out);
                }

                let mut out = Map::new();
                for (key, item) in &map {
                    out.insert(key.clone(), walk(item));
                }
                Value::Object(out)
            }
            other => other,
        }
    }
    walk(&schema)
}
