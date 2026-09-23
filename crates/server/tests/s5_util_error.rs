//! Port of packages/opencode/test/util/error.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: schema-backed named errors serialize to
//! `{ name, data }` objects, and field-less named errors serialize `data: {}`.
#![allow(dead_code)]

use opencode_server::named_error::{AuthError, OutputLengthError};

#[test]
fn schema_backed_named_errors_are_real_named_errors() {
    let error = AuthError::new("anthropic", "boom");

    assert_eq!(
        error.error.to_object(),
        serde_json::json!({
            "name": "ProviderAuthError",
            "data": { "providerID": "anthropic", "message": "boom" },
        })
    );
}

#[test]
fn named_errors_without_fields_serialize_data() {
    let error = OutputLengthError::new();
    assert_eq!(
        error.error.to_object(),
        serde_json::json!({ "name": "MessageOutputLengthError", "data": {} })
    );
}
