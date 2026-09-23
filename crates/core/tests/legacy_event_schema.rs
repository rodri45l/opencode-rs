//! Port of packages/core/test/legacy-event-schema.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: constructing the legacy `APIError` and serializing it with
//! `toObject` yields `{ name: "APIError", data: { message, isRetryable } }`.
//! Re-derived: the cross-package `SessionV1` definition identity assertions are
//! dropped (module identity, not behaviour).

use opencode_core::legacy_event_schema::{api_error_object, ApiError};
use serde_json::json;

const NOTE: &str = "porting: legacy event schema not implemented";

#[test]
fn retains_named_error_constructor_identity() {
    let error = ApiError::new("failed", false).expect(NOTE);
    assert_eq!(
        error,
        ApiError {
            message: "failed".into(),
            is_retryable: false
        }
    );
    assert_eq!(
        error.to_object().expect(NOTE),
        json!({ "name": "APIError", "data": { "message": "failed", "isRetryable": false } })
    );
    assert_eq!(
        api_error_object("failed", false),
        json!({ "name": "APIError", "data": { "message": "failed", "isRetryable": false } })
    );
}
