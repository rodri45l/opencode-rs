//! Named error envelopes for provider/session failures.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/error.ts`'s
//! `NamedError.toObject`: `{ name, data }`, with `data` defaulting to `{}`.

use serde_json::{json, Value};

/// A named error with structured data.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedError {
    /// The stable error name.
    pub name: &'static str,
    /// The structured error payload.
    pub data: Value,
}

impl NamedError {
    /// Serialize to the `{ name, data }` envelope.
    pub fn to_object(&self) -> Value {
        json!({ "name": self.name, "data": self.data })
    }
}

/// Provider authentication failure.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthError {
    /// The serialized named error.
    pub error: NamedError,
}

impl AuthError {
    /// Build a `ProviderAuthError` for `provider_id`.
    pub fn new(provider_id: &str, message: &str) -> AuthError {
        AuthError {
            error: NamedError {
                name: "ProviderAuthError",
                data: json!({ "providerID": provider_id, "message": message }),
            },
        }
    }
}

/// Output-length failure with no extra fields.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputLengthError {
    /// The serialized named error.
    pub error: NamedError,
}

impl OutputLengthError {
    /// Build a `MessageOutputLengthError`.
    pub fn new() -> OutputLengthError {
        OutputLengthError {
            error: NamedError {
                name: "MessageOutputLengthError",
                data: json!({}),
            },
        }
    }
}

impl Default for OutputLengthError {
    fn default() -> Self {
        Self::new()
    }
}
