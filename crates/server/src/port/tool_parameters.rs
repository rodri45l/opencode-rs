//! Tool parameter schemas: accepts/rejects contract.
//!
//! Re-derived from the observable behaviour pinned by
//! `packages/opencode/test/tool/parameters.test.ts` (upstream 18ef3cc). The
//! reference also snapshots the emitted JSON Schema per tool; those snapshots
//! are wire-shape only and are covered by the tool-define tests, so this module
//! pins the accept/reject contract and the pure defaulting rules instead.

use serde_json::{json, Map, Value};

/// A typed parameter-validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamError {
    /// A required field is missing.
    Missing(&'static str),
    /// A field has the wrong type or value.
    Invalid(&'static str),
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(field) => write!(f, "missing required field: {field}"),
            Self::Invalid(field) => write!(f, "invalid field: {field}"),
        }
    }
}

impl std::error::Error for ParamError {}

const LSP_OPERATIONS: &[&str] = &[
    "hover",
    "definition",
    "references",
    "implementation",
    "documentSymbol",
    "workspaceSymbol",
    "goToDefinition",
    "goToImplementation",
    "codeAction",
    "diagnostics",
];

/// Validate `input` against the parameter schema for `tool`.
///
/// Returns the normalized parameters (defaults applied) on success.
pub fn validate(tool: &str, input: &Value) -> Result<Value, ParamError> {
    let object = input
        .as_object()
        .ok_or(ParamError::Invalid("expected object"))?;
    match tool {
        "apply_patch" => {
            require_string(object, "patchText")?;
            Ok(input.clone())
        }
        "shell" => {
            require_string(object, "command")?;
            optional_u64(object, "timeout")?;
            optional_string(object, "workdir")?;
            Ok(input.clone())
        }
        "edit" => {
            require_string(object, "filePath")?;
            require_string(object, "oldString")?;
            require_string(object, "newString")?;
            optional_bool(object, "replaceAll")?;
            Ok(input.clone())
        }
        "glob" => {
            require_string(object, "pattern")?;
            optional_string(object, "path")?;
            Ok(input.clone())
        }
        "grep" => {
            require_string(object, "pattern")?;
            optional_string(object, "path")?;
            optional_string(object, "include")?;
            Ok(input.clone())
        }
        "invalid" => {
            require_string(object, "tool")?;
            require_string(object, "error")?;
            Ok(input.clone())
        }
        "lsp" => {
            let operation = require_string(object, "operation")?;
            if !LSP_OPERATIONS.contains(&operation) {
                return Err(ParamError::Invalid("operation"));
            }
            require_string(object, "filePath")?;
            let line = require_u64(object, "line")?;
            if line < 1 {
                return Err(ParamError::Invalid("line"));
            }
            let character = require_u64(object, "character")?;
            if character < 1 {
                return Err(ParamError::Invalid("character"));
            }
            Ok(input.clone())
        }
        "plan" => Ok(input.clone()),
        "question" => {
            let questions = require_array(object, "questions")?;
            if questions.is_empty() {
                return Err(ParamError::Invalid("questions"));
            }
            Ok(input.clone())
        }
        "read" => {
            require_string(object, "filePath")?;
            optional_u64(object, "offset")?;
            optional_u64(object, "limit")?;
            Ok(input.clone())
        }
        "skill" => {
            require_string(object, "name")?;
            Ok(input.clone())
        }
        "task" => {
            require_string(object, "description")?;
            require_string(object, "prompt")?;
            require_string(object, "subagent_type")?;
            optional_bool(object, "background")?;
            Ok(input.clone())
        }
        "todo" => {
            let todos = require_array(object, "todos")?;
            if todos.is_empty() {
                return Err(ParamError::Invalid("todos"));
            }
            Ok(input.clone())
        }
        "webfetch" => {
            let url = require_string(object, "url")?;
            let format = match object.get("format") {
                None | Some(Value::Null) => "markdown".to_string(),
                Some(Value::String(value)) => match value.as_str() {
                    "text" | "markdown" | "html" => value.clone(),
                    _ => return Err(ParamError::Invalid("format")),
                },
                Some(_) => return Err(ParamError::Invalid("format")),
            };
            let mut normalized = Map::new();
            normalized.insert("url".to_string(), json!(url));
            normalized.insert("format".to_string(), json!(format));
            Ok(Value::Object(normalized))
        }
        "websearch" => {
            require_string(object, "query")?;
            Ok(input.clone())
        }
        "write" => {
            require_string(object, "content")?;
            require_string(object, "filePath")?;
            Ok(input.clone())
        }
        _ => Err(ParamError::Invalid("unknown tool")),
    }
}

fn require_string<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
) -> Result<&'a str, ParamError> {
    match object.get(key) {
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(ParamError::Invalid(key)),
        None => Err(ParamError::Missing(key)),
    }
}

fn optional_string(object: &Map<String, Value>, key: &'static str) -> Result<(), ParamError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(_)) => Ok(()),
        Some(_) => Err(ParamError::Invalid(key)),
    }
}

fn optional_bool(object: &Map<String, Value>, key: &'static str) -> Result<(), ParamError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Bool(_)) => Ok(()),
        Some(_) => Err(ParamError::Invalid(key)),
    }
}

fn require_u64(object: &Map<String, Value>, key: &'static str) -> Result<u64, ParamError> {
    match object.get(key) {
        Some(Value::Number(number)) if number.as_u64().is_some() => {
            Ok(number.as_u64().expect("checked"))
        }
        Some(_) => Err(ParamError::Invalid(key)),
        None => Err(ParamError::Missing(key)),
    }
}

fn optional_u64(object: &Map<String, Value>, key: &'static str) -> Result<(), ParamError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Number(number)) if number.as_u64().is_some() => Ok(()),
        Some(_) => Err(ParamError::Invalid(key)),
    }
}

fn require_array<'a>(
    object: &'a Map<String, Value>,
    key: &'static str,
) -> Result<&'a Vec<Value>, ParamError> {
    match object.get(key) {
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(ParamError::Invalid(key)),
        None => Err(ParamError::Missing(key)),
    }
}
