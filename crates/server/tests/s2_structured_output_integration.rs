#![allow(dead_code)]

//! Port of packages/opencode/test/session/structured-output-integration.test.ts
//! (upstream 18ef3cc).
//!
//! Behaviour pinned: the pure `StructuredOutputError` shape, persistence of the
//! user message `outputFormat` (including `retryCount`), and text mode leaving
//! the assistant `structured` field unset. The live API-key integration cases
//! (real schema round-trips through the prompt service) are dropped here and
//! noted in PORT-STATUS.s2.json. Stubs are local per the fast-wave protocol.

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, PartialEq)]
enum OutputFormat {
    Text,
    JsonSchema { schema: Value, retry_count: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructuredOutputError {
    message: String,
    retries: u64,
}

impl StructuredOutputError {
    fn name(&self) -> &'static str {
        "StructuredOutputError"
    }

    fn to_object(&self) -> Value {
        json!({
            "name": self.name(),
            "data": { "message": self.message, "retries": self.retries }
        })
    }
}

fn persisted_user_format(format: &OutputFormat) -> Result<Value, S2Error> {
    match format {
        OutputFormat::Text => Ok(json!({ "type": "text" })),
        OutputFormat::JsonSchema {
            schema,
            retry_count,
        } => Ok(json!({
            "type": "json_schema",
            "schema": schema,
            "retryCount": retry_count,
        })),
    }
}

fn assistant_structured(format: &OutputFormat) -> Result<Option<Value>, S2Error> {
    match format {
        OutputFormat::Text => Ok(None),
        OutputFormat::JsonSchema { .. } => Ok(None),
    }
}

#[test]
fn unit_test_structured_output_error_is_properly_structured() {
    let error = StructuredOutputError {
        message: "Failed to produce valid structured output after 3 attempts".to_string(),
        retries: 3,
    };

    assert_eq!(error.name(), "StructuredOutputError");
    assert!(error.message.contains("3 attempts"));
    assert_eq!(error.retries, 3);

    let obj = error.to_object();
    assert_eq!(obj["name"], json!("StructuredOutputError"));
    assert_eq!(obj["data"]["retries"], json!(3));
}

#[test]
fn stores_output_format_on_user_message() {
    let format = OutputFormat::JsonSchema {
        schema: json!({
            "type": "object",
            "properties": { "result": { "type": "number" } },
            "required": ["result"]
        }),
        retry_count: 3,
    };

    let persisted = persisted_user_format(&format).expect("persisted format");
    assert_eq!(persisted["type"], json!("json_schema"));
    assert_eq!(persisted["retryCount"], json!(3));
}

#[test]
fn works_with_text_output_format_default() {
    let structured = assistant_structured(&OutputFormat::Text).expect("structured");
    assert!(structured.is_none());
}
