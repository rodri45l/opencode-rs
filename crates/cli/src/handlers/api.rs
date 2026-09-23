//! Request resolution for the `api` command.
//!
//! Mirrors `packages/cli/src/commands/handlers/api.ts`: an operation id is
//! looked up in an OpenAPI document and interpolated into a method + path, or a
//! curl-like `[method, path]` pair is accepted verbatim.

use std::collections::BTreeMap;
use std::fmt;

/// HTTP methods accepted by the generic endpoint constructor.
const METHODS: [&str; 7] = ["delete", "get", "head", "options", "patch", "post", "put"];

/// A resolved outgoing request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRequest {
    pub method: String,
    pub path: String,
}

/// A request-resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// No operation in the document carries the requested id.
    OperationNotFound(String),
    /// A path template placeholder had no matching parameter.
    MissingPathParameter(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::OperationNotFound(id) => write!(f, "Operation not found: {id}"),
            ApiError::MissingPathParameter(name) => {
                write!(f, "Missing path parameter: {name}")
            }
        }
    }
}

impl std::error::Error for ApiError {}

/// A single OpenAPI operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Operation {
    pub operation_id: Option<String>,
}

/// The subset of an OpenAPI document the resolver reads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenApi {
    pub paths: BTreeMap<String, BTreeMap<String, Operation>>,
}

/// Resolve `operation_id` to a method and an interpolated path.
pub fn resolve_operation(
    spec: &OpenApi,
    operation_id: &str,
    params: &[(&str, &str)],
) -> Result<ResolvedRequest, ApiError> {
    for (path, operations) in &spec.paths {
        for (method, operation) in operations {
            if !METHODS.contains(&method.as_str()) {
                continue;
            }
            if operation.operation_id.as_deref() != Some(operation_id) {
                continue;
            }
            return Ok(ResolvedRequest {
                method: method.to_ascii_uppercase(),
                path: interpolate(path, params)?,
            });
        }
    }
    Err(ApiError::OperationNotFound(operation_id.to_string()))
}

/// Resolve a curl-like `[method, path]` pair, if it is well formed.
pub fn raw_request(input: &[String]) -> Option<ResolvedRequest> {
    if input.len() != 2 {
        return None;
    }
    let method = input[0].to_ascii_lowercase();
    if !METHODS.contains(&method.as_str()) {
        return None;
    }
    if !input[1].starts_with('/') {
        return None;
    }
    Some(ResolvedRequest {
        method: input[0].to_ascii_uppercase(),
        path: input[1].clone(),
    })
}

fn interpolate(path: &str, params: &[(&str, &str)]) -> Result<String, ApiError> {
    let mut used: Vec<&str> = Vec::new();
    let mut pathname = String::new();
    let mut rest = path;

    while let Some(open) = rest.find('{') {
        pathname.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            pathname.push_str(&rest[open..]);
            rest = "";
            break;
        };
        let name = &after[..close];
        let value = params
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value);
        match value {
            Some(value) => {
                pathname.push_str(&encode_path_segment(value));
                used.push(name);
            }
            None => return Err(ApiError::MissingPathParameter(name.to_string())),
        }
        rest = &after[close + 1..];
    }
    pathname.push_str(rest);

    let mut query: Vec<String> = Vec::new();
    for (name, value) in params {
        if used.contains(name) {
            continue;
        }
        query.push(format!(
            "{}={}",
            encode_query_component(name),
            encode_query_component(value)
        ));
    }

    if query.is_empty() {
        Ok(pathname)
    } else {
        Ok(format!("{pathname}?{}", query.join("&")))
    }
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
}

fn encode_path_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if is_unreserved(byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn encode_query_component(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte == b' ' {
            out.push('+');
        } else if is_unreserved(byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
