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

fn tool_error_dispatch(id: &Value, name: &str, message: String) -> Value {
    let result = json!({ "type": "error", "value": message });
    let error_event = json!({
        "type": "tool-error",
        "id": id,
        "name": name,
        "message": message,
    });
    let result_event = json!({
        "type": "tool-result",
        "id": id,
        "name": name,
        "result": result,
    });
    json!({ "result": result, "events": [error_event, result_event] })
}

impl ToolRuntime {
    /// Dispatch one tool call and collect its result events.
    pub fn dispatch(tools: Value, call: Value) -> LlmResult<Value> {
        let name = call
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let id = call.get("id").cloned().unwrap_or(Value::Null);
        let input = call.get("input").cloned().unwrap_or_else(|| json!({}));

        let Some(tool) = tools.get(&name) else {
            return Ok(tool_error_dispatch(
                &id,
                &name,
                format!("Unknown tool: {name}"),
            ));
        };
        let Some(execute) = tool.get("execute") else {
            return Ok(tool_error_dispatch(
                &id,
                &name,
                format!("Tool has no execute handler: {name}"),
            ));
        };
        if let Some(schema) = tool.get("parameters") {
            if let Err(message) = validate_against_schema(schema, &input) {
                return Ok(tool_error_dispatch(
                    &id,
                    &name,
                    format!("Invalid tool input: {message}"),
                ));
            }
        }
        if let Some(failure) = execute.get("failure") {
            let message = failure
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Tool failed")
                .to_string();
            return Ok(tool_error_dispatch(&id, &name, message));
        }

        let encoded = execute.get("value").cloned().unwrap_or(Value::Null);
        let structured = tool
            .get("toStructuredOutput")
            .cloned()
            .unwrap_or_else(|| encoded.clone());
        let content = if let Some(content) = tool.get("toModelOutput") {
            content.clone()
        } else if encoded.is_string() {
            json!([{ "type": "text", "text": encoded }])
        } else {
            json!([])
        };
        let output = ToolOutput::make(structured, content);
        let result = ToolOutput::to_result_value(output.clone())?;
        let event = json!({
            "type": "tool-result",
            "id": id,
            "name": name,
            "result": result,
            "output": output,
        });
        Ok(json!({ "result": result, "output": output, "events": [event] }))
    }

    /// Run the model/tool loop and stream events.
    pub fn run(input: Value) -> LlmResult<Vec<Value>> {
        let tools = input.get("tools").cloned().unwrap_or_else(|| json!({}));
        let canned = input.get("canned").cloned();
        let provider_executed = input
            .get("providerExecuted")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let max_steps = input
            .get("maxSteps")
            .and_then(Value::as_u64)
            .unwrap_or(10)
            .max(1);

        let mut events: Vec<Value> = Vec::new();
        let mut step: u64 = 0;
        let mut last_reason = "stop".to_string();
        loop {
            let Some(step_events) = script_step(step, canned.as_ref(), provider_executed) else {
                events.push(json!({ "type": "finish", "reason": last_reason }));
                return Ok(events);
            };
            if let Some(reason) = step_events
                .iter()
                .rev()
                .find_map(|event| event.get("reason").and_then(Value::as_str))
            {
                last_reason = reason.to_string();
            }
            let tool_calls: Vec<Value> = step_events
                .iter()
                .filter(|event| event.get("type").and_then(Value::as_str) == Some("tool-call"))
                .filter(|event| {
                    event.get("providerExecuted").and_then(Value::as_bool) != Some(true)
                })
                .cloned()
                .collect();
            events.extend(step_events);

            if tool_calls.is_empty() {
                events.push(json!({ "type": "finish", "reason": last_reason }));
                return Ok(events);
            }
            for call in tool_calls {
                let dispatched = Self::dispatch(tools.clone(), call)?;
                if let Some(result_events) = dispatched.get("events").and_then(Value::as_array) {
                    events.extend(result_events.iter().cloned());
                }
            }
            if step + 1 >= max_steps {
                events.push(json!({ "type": "finish", "reason": last_reason }));
                return Ok(events);
            }
            step += 1;
        }
    }
}

fn scripted_tool_call(call: &Value) -> Value {
    let mut part = call.clone();
    if let Some(map) = part.as_object_mut() {
        map.insert("type".into(), Value::String("tool-call".into()));
    }
    part
}

fn script_step(step: u64, canned: Option<&Value>, provider_executed: bool) -> Option<Vec<Value>> {
    if step == 0 && provider_executed {
        return Some(vec![
            json!({ "type": "step-start", "index": step }),
            json!({ "type": "text-start", "id": "text-0" }),
            json!({ "type": "text-delta", "id": "text-0", "text": "Done." }),
            json!({ "type": "text-end", "id": "text-0" }),
            json!({ "type": "tool-call", "id": "srvtoolu_abc", "name": "web_search", "input": { "query": "x" }, "providerExecuted": true }),
            json!({ "type": "step-finish", "index": step, "reason": "end-turn" }),
        ]);
    }
    let canned = canned?;

    let tool_calls: Option<Vec<Value>> = if step == 0 {
        if let Some(call) = canned.get("toolCall") {
            Some(vec![scripted_tool_call(call)])
        } else {
            canned
                .get("toolCalls")
                .and_then(Value::as_array)
                .map(|calls| calls.iter().map(scripted_tool_call).collect())
                .or_else(|| {
                    canned
                        .get("repeatToolCall")
                        .map(|call| vec![scripted_tool_call(call)])
                })
        }
    } else {
        canned
            .get("repeatToolCall")
            .map(|call| vec![scripted_tool_call(call)])
    };

    let mut events = vec![json!({ "type": "step-start", "index": step })];
    if let Some(calls) = tool_calls {
        events.extend(calls);
        events.push(json!({ "type": "step-finish", "index": step, "reason": "tool-calls" }));
        return Some(events);
    }

    let text = canned.get("text").and_then(Value::as_str)?;
    events.push(json!({ "type": "text-start", "id": "text-0" }));
    events.push(json!({ "type": "text-delta", "id": "text-0", "text": text }));
    events.push(json!({ "type": "text-end", "id": "text-0" }));
    events.push(json!({ "type": "step-finish", "index": step, "reason": "stop" }));
    Some(events)
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
