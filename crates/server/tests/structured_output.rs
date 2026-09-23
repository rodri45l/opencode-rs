//! Port of packages/opencode/test/session/structured-output.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `OutputFormat` decoding with defaults and validation, the
//! `StructuredOutputError` shape, and the structured-output tool (schema
//! projection, `$schema` stripping, execute/onSuccess, and model output). The
//! UserMessage/AssistantMessage schema cases are dropped (covered by the
//! session schema decoder port).

use opencode_server::structured_output::{
    create_structured_output_tool, parse_output_format, ModelOutput, OutputFormat,
    StructuredOutputError, DEFAULT_RETRY_COUNT,
};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn parses_text_format() {
    let result = parse_output_format(&json!({ "type": "text" })).expect("valid");
    assert_eq!(result, OutputFormat::Text);
}

#[test]
fn parses_json_schema_format_with_defaults() {
    let result = parse_output_format(&json!({
        "type": "json_schema",
        "schema": { "type": "object", "properties": { "name": { "type": "string" } } }
    }))
    .expect("valid");

    match result {
        OutputFormat::JsonSchema { retry_count, .. } => {
            assert_eq!(retry_count, DEFAULT_RETRY_COUNT);
        }
        OutputFormat::Text => panic!("expected json_schema"),
    }
}

#[test]
fn parses_json_schema_format_with_custom_retry_count() {
    let result = parse_output_format(&json!({
        "type": "json_schema",
        "schema": { "type": "object" },
        "retryCount": 5
    }))
    .expect("valid");

    match result {
        OutputFormat::JsonSchema { retry_count, .. } => assert_eq!(retry_count, 5),
        OutputFormat::Text => panic!("expected json_schema"),
    }
}

#[test]
fn rejects_invalid_type() {
    assert!(parse_output_format(&json!({ "type": "invalid" })).is_err());
}

#[test]
fn rejects_json_schema_without_schema() {
    assert!(parse_output_format(&json!({ "type": "json_schema" })).is_err());
}

#[test]
fn rejects_negative_retry_count() {
    assert!(parse_output_format(&json!({
        "type": "json_schema",
        "schema": { "type": "object" },
        "retryCount": -1
    }))
    .is_err());
}

#[test]
fn creates_error_with_message_and_retries() {
    let error = StructuredOutputError::new("Failed to validate", 3);

    assert_eq!(error.name(), "StructuredOutputError");
    assert_eq!(error.message, "Failed to validate");
    assert_eq!(error.retries, 3);
}

#[test]
fn converts_error_to_object() {
    let error = StructuredOutputError::new("Test error", 2);
    let object = error.to_object();

    assert_eq!(object["name"], "StructuredOutputError");
    assert_eq!(object["data"]["message"], "Test error");
    assert_eq!(object["data"]["retries"], 2);
}

#[test]
fn is_instance_identifies_error() {
    let error = StructuredOutputError::new("Test", 1);
    assert!(StructuredOutputError::is_instance(&error.to_object()));
    assert!(!StructuredOutputError::is_instance(
        &json!({ "name": "other" })
    ));
}

#[test]
fn creates_tool_with_description() {
    let tool = create_structured_output_tool(json!({ "type": "object" }), |_| {});
    assert!(tool.description.contains("structured format"));
}

#[test]
fn creates_tool_with_schema_as_input_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "company": { "type": "string" },
            "founded": { "type": "number" }
        },
        "required": ["company"]
    });
    let tool = create_structured_output_tool(schema, |_| {});

    assert!(tool.input_schema["jsonSchema"]["properties"]["company"].is_object());
    assert!(tool.input_schema["jsonSchema"]["properties"]["founded"].is_object());
}

#[test]
fn strips_schema_property_from_input_schema() {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": { "name": { "type": "string" } }
    });
    let tool = create_structured_output_tool(schema, |_| {});

    assert!(tool.input_schema["jsonSchema"].get("$schema").is_none());
}

#[test]
fn execute_calls_on_success_with_valid_args() {
    let captured: Rc<RefCell<Option<serde_json::Value>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&captured);
    let tool = create_structured_output_tool(
        json!({ "type": "object", "properties": { "name": { "type": "string" } } }),
        move |output| *sink.borrow_mut() = Some(output.clone()),
    );

    let args = json!({ "name": "Test Company" });
    let result = tool.execute(args.clone());

    assert_eq!(*captured.borrow(), Some(args));
    assert_eq!(result.output, "Structured output captured successfully.");
    assert_eq!(result.metadata["valid"], true);
}

#[test]
fn validates_required_fields_in_the_projected_schema() {
    let tool = create_structured_output_tool(
        json!({
            "type": "object",
            "properties": { "name": { "type": "string" }, "age": { "type": "number" } },
            "required": ["name", "age"]
        }),
        |_| {},
    );

    let required = tool.input_schema["jsonSchema"]["required"]
        .as_array()
        .expect("required");
    assert!(required.iter().any(|value| value == "name"));
    assert!(required.iter().any(|value| value == "age"));
}

#[test]
fn preserves_property_types_in_the_projected_schema() {
    let tool = create_structured_output_tool(
        json!({
            "type": "object",
            "properties": { "count": { "type": "number" } },
            "required": ["count"]
        }),
        |_| {},
    );

    assert_eq!(
        tool.input_schema["jsonSchema"]["properties"]["count"]["type"],
        "number"
    );
}

#[test]
fn execute_handles_nested_objects() {
    let captured: Rc<RefCell<Option<serde_json::Value>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&captured);
    let tool = create_structured_output_tool(
        json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "email": { "type": "string" }
                    },
                    "required": ["name"]
                }
            },
            "required": ["user"]
        }),
        move |output| *sink.borrow_mut() = Some(output.clone()),
    );

    let args = json!({ "user": { "name": "John", "email": "john@test.com" } });
    let result = tool.execute(args.clone());

    assert_eq!(*captured.borrow(), Some(args));
    assert_eq!(result.metadata["valid"], true);
    assert_eq!(
        tool.input_schema["jsonSchema"]["properties"]["user"]["type"],
        "object"
    );
    assert_eq!(
        tool.input_schema["jsonSchema"]["properties"]["user"]["properties"]["name"]["type"],
        "string"
    );
}

#[test]
fn execute_handles_arrays() {
    let captured: Rc<RefCell<Option<serde_json::Value>>> = Rc::new(RefCell::new(None));
    let sink = Rc::clone(&captured);
    let tool = create_structured_output_tool(
        json!({
            "type": "object",
            "properties": { "tags": { "type": "array", "items": { "type": "string" } } },
            "required": ["tags"]
        }),
        move |output| *sink.borrow_mut() = Some(output.clone()),
    );

    let args = json!({ "tags": ["a", "b", "c"] });
    let result = tool.execute(args.clone());

    assert_eq!(*captured.borrow(), Some(args));
    assert_eq!(result.metadata["valid"], true);
    assert_eq!(
        tool.input_schema["jsonSchema"]["properties"]["tags"]["type"],
        "array"
    );
    assert_eq!(
        tool.input_schema["jsonSchema"]["properties"]["tags"]["items"]["type"],
        "string"
    );
}

#[test]
fn to_model_output_returns_text_value() {
    let tool = create_structured_output_tool(json!({ "type": "object" }), |_| {});
    let output = tool.to_model_output("Test output");
    assert_eq!(output, ModelOutput::Text("Test output".to_string()));
}
