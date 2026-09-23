//! Port of packages/opencode/test/acp/error.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/error.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure error mapping `toRequestError` / `fromUnknownDefect`.
//! Dropped: none — every upstream case is pure.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

#[derive(Debug, Clone, PartialEq)]
enum AcpError {
    SessionNotFound {
        session_id: String,
    },
    InvalidConfigOption {
        config_id: String,
    },
    InvalidModel {
        provider_id: String,
        model_id: String,
    },
    InvalidEffort {
        effort: String,
    },
    InvalidMode {
        mode: String,
    },
    AuthRequired {
        provider_id: String,
    },
    UnsupportedOperation {
        method: String,
    },
    ServiceFailure {
        service: String,
        safe_message: String,
    },
    UnknownDefect,
}

#[derive(Debug, Clone, PartialEq)]
struct RequestError {
    code: i64,
    message: String,
    data: Value,
}

impl RequestError {
    fn to_error_response(&self) -> Value {
        json!({ "code": self.code, "message": self.message, "data": self.data })
    }
}

fn to_request_error(_error: AcpError) -> Result<RequestError, NotImplemented> {
    nope("acp error")
}

fn from_unknown_defect(_detail: &str) -> AcpError {
    AcpError::UnknownDefect
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn maps_validation_failures_to_invalid_params() {
    let cases = vec![
        AcpError::SessionNotFound {
            session_id: "ses_missing".into(),
        },
        AcpError::InvalidConfigOption {
            config_id: "temperature".into(),
        },
        AcpError::InvalidModel {
            provider_id: "anthropic".into(),
            model_id: "claude-missing".into(),
        },
        AcpError::InvalidEffort {
            effort: "extreme".into(),
        },
        AcpError::InvalidMode {
            mode: "turbo".into(),
        },
    ];
    let codes: Vec<i64> = cases
        .into_iter()
        .map(|error| to_request_error(error).unwrap().code)
        .collect();
    assert_eq!(codes, vec![-32602, -32602, -32602, -32602, -32602]);
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn includes_safe_validation_details() {
    let session = to_request_error(AcpError::SessionNotFound {
        session_id: "ses_123".into(),
    })
    .unwrap();
    assert_eq!(session.code, -32602);
    assert_eq!(session.data, json!({ "sessionId": "ses_123" }));
    let model = to_request_error(AcpError::InvalidModel {
        provider_id: "anthropic".into(),
        model_id: "gpt-missing".into(),
    })
    .unwrap();
    assert_eq!(model.code, -32602);
    assert_eq!(model.data, json!({ "modelId": "gpt-missing" }));
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn maps_auth_required_to_the_sdk_auth_error() {
    let request_error = to_request_error(AcpError::AuthRequired {
        provider_id: "anthropic".into(),
    })
    .unwrap();
    assert_eq!(request_error.code, -32000);
    assert_eq!(
        request_error.message,
        "Authentication required: provider authentication required"
    );
    assert_eq!(request_error.data, json!({ "providerId": "anthropic" }));
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn maps_unsupported_operations_to_method_not_found() {
    let request_error = to_request_error(AcpError::UnsupportedOperation {
        method: "session/new".into(),
    })
    .unwrap();
    assert_eq!(request_error.code, -32601);
    assert_eq!(request_error.data, json!({ "method": "session/new" }));
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn maps_service_failures_to_safe_internal_errors() {
    let request_error = to_request_error(AcpError::ServiceFailure {
        service: "provider".into(),
        safe_message: "Provider request failed".into(),
    })
    .unwrap();
    assert_eq!(request_error.code, -32603);
    assert_eq!(
        request_error.message,
        "Internal error: Provider request failed"
    );
    assert_eq!(request_error.data, json!({ "service": "provider" }));
}

#[test]
#[ignore = "porting: acp error not implemented"]
fn wraps_unknown_defects_without_leaking_raw_details() {
    let request_error = to_request_error(from_unknown_defect(
        "stack has sk-ant-secret and oauth refresh token",
    ))
    .unwrap();
    let serialized = request_error.to_error_response().to_string();
    assert_eq!(request_error.code, -32603);
    assert_eq!(
        request_error.message,
        "Internal error: Internal service failure"
    );
    assert!(!serialized.contains("sk-ant-secret"));
    assert!(!serialized.contains("oauth refresh token"));
    assert!(!serialized.contains("stack"));
}
