//! Structured output formats and the structured-output tool.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/prompt.ts`
//! (`OutputFormat` decoding, `StructuredOutputError`, and
//! `createStructuredOutputTool`) against the Rust API.

use serde_json::{json, Value};

use crate::tools::ToolResult;

/// The default retry count for `json_schema` output.
pub const DEFAULT_RETRY_COUNT: i64 = 2;

/// A structured-output request format.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// Plain text output.
    Text,
    /// JSON-schema-constrained output.
    JsonSchema {
        /// The JSON schema.
        schema: Value,
        /// Number of validation retries.
        retry_count: i64,
    },
}

/// A format decoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    /// The diagnostic message.
    pub message: String,
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FormatError {}

/// Decode an output format from its wire JSON.
pub fn parse_output_format(value: &Value) -> Result<OutputFormat, FormatError> {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| FormatError {
            message: "output format requires a type".to_string(),
        })?;
    match kind {
        "text" => Ok(OutputFormat::Text),
        "json_schema" => {
            let schema = value.get("schema").cloned().ok_or_else(|| FormatError {
                message: "json_schema output format requires a schema".to_string(),
            })?;
            let retry_count = value
                .get("retryCount")
                .and_then(Value::as_i64)
                .unwrap_or(DEFAULT_RETRY_COUNT);
            if retry_count < 0 {
                return Err(FormatError {
                    message: "retryCount must not be negative".to_string(),
                });
            }
            Ok(OutputFormat::JsonSchema {
                schema,
                retry_count,
            })
        }
        other => Err(FormatError {
            message: format!("unknown output format type: {other}"),
        }),
    }
}

/// The error raised when structured output cannot be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredOutputError {
    /// The diagnostic message.
    pub message: String,
    /// The number of retries attempted.
    pub retries: u32,
}

impl StructuredOutputError {
    /// Construct the error.
    pub fn new(message: impl Into<String>, retries: u32) -> Self {
        Self {
            message: message.into(),
            retries,
        }
    }

    /// The stable error name.
    pub fn name(&self) -> &'static str {
        "StructuredOutputError"
    }

    /// Whether a wire value is a structured-output error.
    pub fn is_instance(value: &Value) -> bool {
        value.get("name").and_then(Value::as_str) == Some("StructuredOutputError")
    }

    /// The wire object form.
    pub fn to_object(&self) -> Value {
        json!({
            "name": self.name(),
            "data": {
                "message": self.message,
                "retries": self.retries,
            },
        })
    }
}

/// A text model output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelOutput {
    /// A text payload.
    Text(String),
}

/// The tool the prompt loop exposes for structured output.
pub struct StructuredOutputTool {
    /// Description shown to the model.
    pub description: String,
    /// The AI-SDK-shaped input schema.
    pub input_schema: Value,
    on_success: Box<dyn Fn(&Value)>,
}

impl StructuredOutputTool {
    /// Execute the tool, invoking `on_success` with the decoded output.
    pub fn execute(&self, args: Value) -> ToolResult {
        (self.on_success)(&args);
        ToolResult {
            title: "structured output".to_string(),
            output: "Structured output captured successfully.".to_string(),
            metadata: json!({ "valid": true }),
            attachments: None,
        }
    }

    /// Render the tool output back to the model.
    pub fn to_model_output(&self, output: &str) -> ModelOutput {
        ModelOutput::Text(output.to_string())
    }
}

/// Build the structured-output tool for `schema`.
pub fn create_structured_output_tool(
    schema: Value,
    on_success: impl Fn(&Value) + 'static,
) -> StructuredOutputTool {
    let mut cleaned = schema;
    if let Some(object) = cleaned.as_object_mut() {
        object.remove("$schema");
    }
    StructuredOutputTool {
        description: "Capture the response in the requested structured format.".to_string(),
        input_schema: json!({ "jsonSchema": cleaned }),
        on_success: Box::new(on_success),
    }
}
