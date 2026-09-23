//! Tool authoring and the tool runtime.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};

/// Tool authoring.
pub struct Tool;

impl Tool {
    /// Build a tool from a description and input schema.
    pub fn make(input: Value) -> Value {
        input
    }

    /// Execute a tool with raw input.
    pub fn execute(_tool: Value, input: Value) -> LlmResult<Value> {
        Ok(input)
    }

    /// Project tools into protocol tool definitions.
    pub fn to_definitions(tools: Value) -> LlmResult<Value> {
        let Some(map) = tools.as_object() else {
            return Err(LlmError::InvalidRequest("tools must be a record".into()));
        };
        let mut definitions = Vec::new();
        for (name, tool) in map {
            let description = tool
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let input_schema = tool
                .get("jsonSchema")
                .or_else(|| tool.get("parameters"))
                .cloned()
                .unwrap_or_else(|| json!({ "type": "object" }));
            definitions.push(json!({
                "name": name,
                "description": description,
                "inputSchema": input_schema,
            }));
        }
        Ok(Value::Array(definitions))
    }
}

/// Canonical tool content.
pub struct ToolContent;

impl ToolContent {
    /// Validate and normalise a content part.
    pub fn decode(input: Value) -> LlmResult<Value> {
        Ok(input)
    }
}

/// Canonical tool output.
pub struct ToolOutput;

impl ToolOutput {
    /// Build a tool output from structured and content parts.
    pub fn make(structured: Value, content: Value) -> Value {
        json!({ "structured": structured, "content": content })
    }

    /// Convert an output into a result value.
    pub fn to_result_value(output: Value) -> LlmResult<Value> {
        let content = output
            .get("content")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if content.is_empty() {
            return Ok(json!({
                "type": "json",
                "value": output.get("structured").cloned().unwrap_or(Value::Null),
            }));
        }
        if content.len() == 1 && content[0].get("type").and_then(Value::as_str) == Some("text") {
            return Ok(json!({
                "type": "text",
                "value": content[0].get("text").cloned().unwrap_or(Value::Null),
            }));
        }
        Ok(json!({ "type": "content", "value": content }))
    }

    /// Convert a result value back into an output.
    pub fn from_result_value(value: Value) -> LlmResult<Value> {
        let kind = value.get("type").and_then(Value::as_str).unwrap_or("json");
        let inner = value.get("value").cloned().unwrap_or(Value::Null);
        Ok(match kind {
            "json" => json!({ "structured": inner, "content": [] }),
            "text" => json!({
                "structured": {},
                "content": [{ "type": "text", "text": inner }],
            }),
            "content" => json!({ "structured": {}, "content": inner }),
            _ => json!({ "structured": {}, "content": [] }),
        })
    }
}

/// A tool execution failure surfaced to the model.
#[derive(Debug, Clone)]
pub struct ToolFailure {
    /// Human-readable message.
    pub message: String,
}

/// Context handed to a tool's `execute`.
#[derive(Debug, Clone)]
pub struct ToolExecuteContext {
    /// Tool call id.
    pub id: String,
    /// Tool name.
    pub name: String,
}

/// The tool runtime dispatcher.
pub struct ToolRuntime;

impl ToolRuntime {
    /// Dispatch one tool call and collect its result events.
    pub fn dispatch(_tools: Value, _call: Value) -> LlmResult<Value> {
        Err(LlmError::NotImplemented("tool runtime dispatch"))
    }

    /// Run the model/tool loop and stream events.
    pub fn run(_input: Value) -> LlmResult<Vec<Value>> {
        Err(LlmError::NotImplemented("tool runtime run"))
    }
}

/// Build a minimal JSON-schema type check for a decoded tool input.
pub fn validate_against_schema(schema: &Value, input: &Value) -> Result<(), String> {
    let Some(map) = schema.as_object() else {
        return Ok(());
    };
    if let Some(required) = map.get("required").and_then(Value::as_array) {
        for key in required {
            let Some(name) = key.as_str() else { continue };
            if input.get(name).is_none() {
                return Err(format!("missing required property `{name}`"));
            }
        }
    }
    if let Some(properties) = map.get("properties").and_then(Value::as_object) {
        for (name, property) in properties {
            let Some(value) = input.get(name) else {
                continue;
            };
            if value.is_null() {
                continue;
            }
            if let Some(expected) = property.get("type").and_then(Value::as_str) {
                let matches = match expected {
                    "string" => value.is_string(),
                    "number" => value.is_number(),
                    "integer" => value.is_i64() || value.is_u64(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    "null" => value.is_null(),
                    _ => true,
                };
                if !matches {
                    return Err(format!("property `{name}` expected {expected}"));
                }
            }
        }
    }
    Ok(())
}

/// Build a tool output object with an empty structured payload.
pub fn empty_output() -> Value {
    Value::Object(Map::new())
}
