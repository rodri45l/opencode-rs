//! Error projection helpers.
//!
//! Port of packages/tui/src/util/error.ts `errorMessage`/`errorFormat`/`errorData`
//! behaviour (upstream 18ef3cc).

/// A host-neutral stand-in for an arbitrary thrown value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorValue {
    /// An `Error`-like value with a name and message.
    Native { name: String, message: String },
    /// A record carrying a message and optional code.
    Record {
        message: String,
        code: Option<String>,
    },
    /// An opaque object whose own properties are not enumerable.
    Opaque { name: String },
    /// A value with a custom `toString`.
    Custom { text: String },
}

/// A projected error payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorData {
    pub type_name: String,
    pub message: String,
    pub formatted: String,
    pub code: Option<String>,
}

/// The message of an error value.
pub fn error_message(error: &ErrorValue) -> String {
    match error {
        ErrorValue::Native { name, message } => {
            if !message.is_empty() {
                message.clone()
            } else {
                name.clone()
            }
        }
        ErrorValue::Record { message, .. } => message.clone(),
        ErrorValue::Opaque { name } => name.clone(),
        ErrorValue::Custom { text } => text.clone(),
    }
}

/// A printable representation of an error value.
pub fn error_format(error: &ErrorValue) -> String {
    match error {
        ErrorValue::Native { name, message } => format!("{name}: {message}"),
        ErrorValue::Record { message, .. } => message.clone(),
        ErrorValue::Opaque { name } => format!("{name} (no message)"),
        ErrorValue::Custom { text } => text.clone(),
    }
}

/// The structured data of an error value.
pub fn error_data(error: &ErrorValue) -> ErrorData {
    match error {
        ErrorValue::Native { name, .. } => ErrorData {
            type_name: name.clone(),
            message: error_message(error),
            formatted: error_format(error),
            code: None,
        },
        ErrorValue::Record { message, code } => ErrorData {
            type_name: "Error".to_string(),
            message: message.clone(),
            formatted: error_format(error),
            code: code.clone(),
        },
        ErrorValue::Opaque { name } => ErrorData {
            type_name: name.clone(),
            message: error_message(error),
            formatted: error_format(error),
            code: None,
        },
        ErrorValue::Custom { text } => ErrorData {
            type_name: "Error".to_string(),
            message: text.clone(),
            formatted: error_format(error),
            code: None,
        },
    }
}
