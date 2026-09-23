//! JSON Schema to TypeScript signature rendering.
//!
//! Port of `packages/codemode/src/tool-schema.ts` (upstream 18ef3cc): schema
//! descriptions and constraints surface as JSDoc, non-identifier keys are
//! quoted, unions and intersections are preserved, and cyclic/deep schemas
//! degrade to `unknown` instead of overflowing.

use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

/// A tool whose input/output schemas are raw JSON Schema documents.
#[derive(Debug, Clone)]
pub struct Tool {
    pub input: Value,
    pub output: Option<Value>,
}

const MAX_RENDER_DEPTH: usize = 8;

/// Render a raw JSON Schema document as a TypeScript type string.
pub fn json_schema_to_type_script(schema: &Value, pretty: bool) -> String {
    let definitions = merge_definitions(BTreeMap::new(), schema);
    render_schema(schema, &definitions, pretty, 0, &HashSet::new())
}

/// The model-visible TypeScript type of a tool's input.
pub fn input_type_script(tool: &Tool, pretty: bool) -> String {
    json_schema_to_type_script(&tool.input, pretty)
}

/// The model-visible TypeScript type of a tool's result.
pub fn output_type_script(tool: &Tool, pretty: bool) -> String {
    match &tool.output {
        Some(output) => json_schema_to_type_script(output, pretty),
        None => "unknown".to_string(),
    }
}

fn merge_definitions(mut base: BTreeMap<String, Value>, schema: &Value) -> BTreeMap<String, Value> {
    for key in ["definitions", "$defs"] {
        if let Some(Value::Object(map)) = schema.get(key) {
            for (name, value) in map {
                base.insert(name.clone(), value.clone());
            }
        }
    }
    base
}

fn render_literal(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string())
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' || first == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn render_key(name: &str) -> String {
    if is_identifier(name) {
        name.to_string()
    } else {
        serde_json::to_string(name).unwrap_or_else(|_| format!("\"{name}\""))
    }
}

fn effect_number_sentinel(schema: &Value) -> bool {
    if schema.get("type").and_then(Value::as_str) != Some("string") {
        return false;
    }
    match schema.get("enum").and_then(Value::as_array) {
        Some(values) if values.len() == 1 => matches!(
            values[0].as_str(),
            Some("NaN") | Some("Infinity") | Some("-Infinity")
        ),
        _ => false,
    }
}

fn intersection(members: &[String]) -> String {
    let concrete: Vec<&String> = members
        .iter()
        .filter(|member| member.as_str() != "unknown")
        .collect();
    match concrete.len() {
        0 => "unknown".to_string(),
        1 => concrete[0].clone(),
        _ => concrete
            .iter()
            .map(|member| {
                if member.contains(" | ") {
                    format!("({member})")
                } else {
                    (*member).clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" & "),
    }
}

fn doc_tags(schema: &Value) -> Vec<String> {
    let mut tags = Vec::new();
    if schema.get("deprecated").and_then(Value::as_bool) == Some(true) {
        tags.push("@deprecated".to_string());
    }
    if let Some(default) = schema.get("default") {
        if let Ok(rendered) = serde_json::to_string(default) {
            tags.push(format!("@default {rendered}"));
        }
    }
    if let Some(format) = schema.get("format").and_then(Value::as_str) {
        tags.push(format!("@format {format}"));
    }
    if let Some(min) = schema.get("minItems").and_then(Value::as_i64) {
        tags.push(format!("@minItems {min}"));
    }
    if let Some(max) = schema.get("maxItems").and_then(Value::as_i64) {
        tags.push(format!("@maxItems {max}"));
    }
    tags
}

fn jsdoc(description: Option<&str>, tags: &[String], pad: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    if let Some(description) = description {
        for line in description.split('\n') {
            lines.push(line.to_string());
        }
    }
    lines.extend(tags.iter().cloned());
    let mut lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.replace("*/", "* /").trim_end().to_string())
        .collect();
    while lines
        .first()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        lines.remove(0);
    }
    while lines
        .last()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        lines.pop();
    }
    if lines.is_empty() {
        return String::new();
    }
    if lines.len() == 1 {
        return format!("{pad}/** {} */\n", lines[0]);
    }
    let body = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                format!("{pad} *")
            } else {
                format!("{pad} * {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{pad}/**\n{body}\n{pad} */\n")
}

fn ref_name(reference: &str) -> Option<String> {
    let rest = reference
        .strip_prefix("#/$defs/")
        .or_else(|| reference.strip_prefix("#/definitions/"))?;
    if rest.contains('/') {
        return None;
    }
    Some(rest.replace("~1", "/").replace("~0", "~"))
}

fn has_unresolved_ref(
    schema: &Value,
    definitions: &BTreeMap<String, Value>,
    seen: &HashSet<String>,
) -> bool {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        match ref_name(reference).and_then(|name| definitions.get(&name).map(|def| (name, def))) {
            Some((name, definition)) if !seen.contains(&name) => {
                let mut next = seen.clone();
                next.insert(name);
                if has_unresolved_ref(definition, definitions, &next) {
                    return true;
                }
            }
            _ => return true,
        }
    }
    for key in ["anyOf", "oneOf", "allOf"] {
        if let Some(items) = schema.get(key).and_then(Value::as_array) {
            if items
                .iter()
                .any(|item| has_unresolved_ref(item, definitions, seen))
            {
                return true;
            }
        }
    }
    if let Some(Value::Object(properties)) = schema.get("properties") {
        if properties
            .values()
            .any(|value| has_unresolved_ref(value, definitions, seen))
        {
            return true;
        }
    }
    if let Some(items) = schema.get("items") {
        if has_unresolved_ref(items, definitions, seen) {
            return true;
        }
    }
    if let Some(additional) = schema.get("additionalProperties") {
        if additional.is_object() && has_unresolved_ref(additional, definitions, seen) {
            return true;
        }
    }
    false
}

fn ordered_properties(map: &serde_json::Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut indexed: Vec<(&String, &Value)> = Vec::new();
    let mut plain: Vec<(&String, &Value)> = Vec::new();
    for (name, value) in map {
        if is_array_index(name) {
            indexed.push((name, value));
        } else {
            plain.push((name, value));
        }
    }
    indexed.sort_by_key(|(name, _)| name.parse::<u64>().unwrap_or(u64::MAX));
    indexed.extend(plain);
    indexed
}

fn is_array_index(name: &str) -> bool {
    if name.is_empty() || name.len() > 10 {
        return false;
    }
    if name != "0" && name.starts_with('0') {
        return false;
    }
    let Ok(value) = name.parse::<u64>() else {
        return false;
    };
    value < 4_294_967_295
}

fn render_schema(
    schema: &Value,
    definitions: &BTreeMap<String, Value>,
    pretty: bool,
    depth: usize,
    seen: &HashSet<String>,
) -> String {
    if depth > MAX_RENDER_DEPTH {
        return "unknown".to_string();
    }
    let nested = merge_definitions(definitions.clone(), schema);

    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        let name = ref_name(reference);
        let resolved = name
            .as_ref()
            .and_then(|name| nested.get(name).map(|definition| (name, definition)));
        return match resolved {
            Some((name, definition)) if !seen.contains(name) => {
                let mut next = seen.clone();
                next.insert(name.clone());
                let mut sibling = schema.clone();
                if let Some(object) = sibling.as_object_mut() {
                    object.remove("$ref");
                }
                intersection(&[
                    render_schema(definition, &nested, pretty, depth, &next),
                    render_schema(&sibling, &nested, pretty, depth + 1, seen),
                ])
            }
            _ => "unknown".to_string(),
        };
    }

    if let Some(constant) = schema.get("const") {
        return render_literal(constant);
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return values
            .iter()
            .map(render_literal)
            .collect::<Vec<_>>()
            .join(" | ");
    }

    let alternatives = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array);
    if let Some(alternatives) = alternatives {
        if alternatives
            .iter()
            .any(|item| item.get("type").and_then(Value::as_str) == Some("number"))
            && alternatives.iter().all(|item| {
                item.get("type").and_then(Value::as_str) == Some("number")
                    || effect_number_sentinel(item)
            })
        {
            return "number".to_string();
        }
        let empty_shape = alternatives.len() == 2
            && alternatives[0].get("type").and_then(Value::as_str) == Some("object")
            && alternatives[0].get("properties").is_none()
            && alternatives[1].get("type").and_then(Value::as_str) == Some("array")
            && alternatives[1].get("items").is_none();
        if empty_shape {
            return "{}".to_string();
        }
        let members: Vec<String> = alternatives
            .iter()
            .map(|item| render_schema(item, &nested, pretty, depth + 1, seen))
            .collect();
        if members.iter().any(|member| member == "unknown") {
            return "unknown".to_string();
        }
        let mut remainder = schema.clone();
        if let Some(object) = remainder.as_object_mut() {
            object.remove("anyOf");
            object.remove("oneOf");
        }
        return intersection(&[
            members.join(" | "),
            render_schema(&remainder, &nested, pretty, depth + 1, seen),
        ]);
    }

    if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
        if all_of
            .iter()
            .any(|item| has_unresolved_ref(item, &nested, &HashSet::new()))
        {
            return "unknown".to_string();
        }
        let members: Vec<String> = all_of
            .iter()
            .map(|item| render_schema(item, &nested, pretty, depth + 1, seen))
            .collect();
        let mut remainder = schema.clone();
        if let Some(object) = remainder.as_object_mut() {
            object.remove("allOf");
        }
        let mut combined = vec![render_schema(&remainder, &nested, pretty, depth + 1, seen)];
        combined.extend(members);
        return intersection(&combined);
    }

    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        return types
            .iter()
            .map(|item| {
                let mut variant = schema.clone();
                if let Some(object) = variant.as_object_mut() {
                    object.insert("type".to_string(), item.clone());
                }
                render_schema(&variant, &nested, pretty, depth + 1, seen)
            })
            .collect::<Vec<_>>()
            .join(" | ");
    }

    match schema.get("type").and_then(Value::as_str) {
        Some("string") => return "string".to_string(),
        Some("number") | Some("integer") => return "number".to_string(),
        Some("boolean") => return "boolean".to_string(),
        Some("null") => return "null".to_string(),
        Some("array") => {
            let items = schema.get("items").cloned().unwrap_or(Value::Null);
            return format!(
                "Array<{}>",
                render_schema(&items, &nested, pretty, depth + 1, seen)
            );
        }
        Some("object") => {}
        _ if schema.get("properties").is_some() => {}
        _ => return "unknown".to_string(),
    }

    let required: HashSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let index_type = schema
        .get("additionalProperties")
        .filter(|additional| additional.is_object())
        .map(|additional| render_schema(additional, &nested, pretty, depth + 1, seen));
    let entries = ordered_properties(&properties);
    let field = |name: &str, value: &Value| {
        format!(
            "{}{}: {}",
            render_key(name),
            if required.contains(name) { "" } else { "?" },
            render_schema(value, &nested, pretty, depth + 1, seen)
        )
    };

    if !pretty {
        let mut fields: Vec<String> = entries
            .iter()
            .map(|(name, value)| field(name, value))
            .collect();
        if let Some(index_type) = &index_type {
            fields.push(format!("[key: string]: {index_type}"));
        }
        return if fields.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", fields.join("; "))
        };
    }

    if entries.is_empty() && index_type.is_none() {
        return "{}".to_string();
    }
    let pad = "  ".repeat(depth + 1);
    let mut lines: Vec<String> = entries
        .iter()
        .map(|(name, value)| {
            let comment = jsdoc(
                value.get("description").and_then(Value::as_str),
                &doc_tags(value),
                &pad,
            );
            format!("{comment}{pad}{},", field(name, value))
        })
        .collect();
    if let Some(index_type) = &index_type {
        lines.push(format!("{pad}[key: string]: {index_type},"));
    }
    format!("{{\n{}\n{}}}", lines.join("\n"), "  ".repeat(depth))
}
