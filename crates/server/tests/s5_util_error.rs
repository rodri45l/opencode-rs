//! Port of packages/opencode/test/util/error.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: schema-backed named errors serialize to
//! `{ name, data }` objects, and field-less named errors serialize `data: {}`.
#![allow(dead_code)]

// Fast-wave local stubs: `session::message_error` is not implemented in this
// crate yet, so the named-error surface is defined here.
mod message_error {
    #[derive(Debug, Clone, PartialEq)]
    pub struct NamedError {
        pub name: &'static str,
        pub data: serde_json::Value,
    }

    impl NamedError {
        pub fn to_object(&self) -> serde_json::Value {
            serde_json::json!({ "name": self.name, "data": self.data })
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct AuthError {
        pub error: NamedError,
    }

    impl AuthError {
        pub fn new(_provider_id: &str, _message: &str) -> Result<AuthError, &'static str> {
            Err("porting: MessageError.AuthError not implemented")
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OutputLengthError {
        pub error: NamedError,
    }

    impl OutputLengthError {
        pub fn new() -> Result<OutputLengthError, &'static str> {
            Err("porting: MessageError.OutputLengthError not implemented")
        }
    }
}

#[test]
#[ignore = "porting: message-error not implemented"]
fn schema_backed_named_errors_are_real_named_errors() {
    let error = message_error::AuthError::new("anthropic", "boom").unwrap();

    assert_eq!(
        error.error.to_object(),
        serde_json::json!({
            "name": "ProviderAuthError",
            "data": { "providerID": "anthropic", "message": "boom" },
        })
    );
}

#[test]
#[ignore = "porting: message-error not implemented"]
fn named_errors_without_fields_serialize_data() {
    let error = message_error::OutputLengthError::new().unwrap();
    assert_eq!(
        error.error.to_object(),
        serde_json::json!({ "name": "MessageOutputLengthError", "data": {} })
    );
}
