//! `LLM.generateObject` — force a synthetic tool call and decode its input.

use serde_json::{json, Value};

use crate::error::{LlmError, LlmResult};
use crate::schema::build_request;
use crate::tool::{validate_against_schema, Tool};

const TOOL_NAME: &str = "generate_object";

/// Force a synthetic tool call and decode the structured result.
pub fn generate_object(input: Value) -> LlmResult<Value> {
    let request = build_request(input)?;
    let schema = request
        .get("schema")
        .or_else(|| request.get("jsonSchema"))
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object" }));
    let tool = Tool::make(json!({
        "description": "Return the structured result by calling this tool.",
        "jsonSchema": schema,
    }));
    let definitions = Tool::to_definitions(json!({ TOOL_NAME: tool }))?;

    let mut generate_request = request.clone();
    generate_request["tools"] = definitions;
    generate_request["toolChoice"] = json!({ "type": "tool", "name": TOOL_NAME });

    let prepared = crate::engine::prepare(generate_request.clone())?;
    let events = crate::engine::stream(generate_request)?;
    let response = crate::engine::reduce_response(events)?;

    let call = response
        .tool_calls
        .iter()
        .find(|call| call.get("name").and_then(Value::as_str) == Some(TOOL_NAME))
        .cloned();
    let Some(call) = call else {
        return Err(LlmError::Provider(format!(
            "generateObject: model did not call the forced `{TOOL_NAME}` tool"
        )));
    };
    let object = call.get("input").cloned().unwrap_or(Value::Null);
    validate_against_schema(&schema, &object).map_err(|error| {
        LlmError::Provider(format!(
            "generateObject: tool input failed schema decode: {error}"
        ))
    })?;

    Ok(json!({
        "object": object,
        "bodies": [prepared.body],
        "response": {
            "toolCalls": response.tool_calls,
            "events": response.events,
            "usage": response.usage,
            "finishReason": response.finish_reason,
        }
    }))
}
