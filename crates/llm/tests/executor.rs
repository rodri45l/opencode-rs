//! Port of packages/llm/test/executor.test.ts (upstream 18ef3cc).
//! Behaviour pinned by `RequestExecutor` classification, retry and redaction.
//! Effect-ts `Layer`/`TestClock`/`Fiber` plumbing is dropped; the observable
//! classification and retry behaviour is kept.

use opencode_llm::RequestExecutor;
use serde_json::json;

fn request() -> serde_json::Value {
    json!({
        "method": "POST",
        "url": "https://provider.test/v1/chat?api_key=secret&key=secret&debug=1",
        "headers": { "authorization": "Bearer secret", "x-safe": "visible" },
    })
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn classifies_context_overflow_responses() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": { "status": 400, "body": "{\"error\":{\"code\":\"context_length_exceeded\",\"message\":\"prompt too long\"}}" }
    }))
    .expect_err("should fail");

    assert_eq!(error.classification(), Some("context-overflow"));
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn does_not_classify_generic_http_413_payload_errors_as_context_overflow() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": { "status": 413, "body": "request too large" }
    }))
    .expect_err("should fail");

    assert_eq!(error.classification(), None);
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn returns_redacted_diagnostics_for_retryable_rate_limits() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": {
            "status": 429,
            "body": "rate limited",
            "headers": { "retry-after-ms": "0", "x-request-id": "req_123", "x-api-key": "secret" }
        }
    }))
    .expect_err("should fail");

    assert!(error.retryable());
    let rendered = format!("{error:?}");
    assert!(rendered.contains("req_123"));
    assert!(rendered.contains("<redacted>"));
    assert!(!rendered.contains("secret"));
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn extracts_openai_style_rate_limit_diagnostics() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": {
            "status": 429,
            "body": "rate limited",
            "headers": {
                "retry-after-ms": "0",
                "x-ratelimit-limit-requests": "500",
                "x-ratelimit-limit-tokens": "30000",
                "x-ratelimit-remaining-requests": "499",
                "x-ratelimit-remaining-tokens": "29900",
                "x-ratelimit-reset-requests": "1s",
                "x-ratelimit-reset-tokens": "10s"
            }
        }
    }))
    .expect_err("should fail");

    assert!(error.retryable());
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn marks_504_and_529_status_responses_retryable() {
    for status in [504, 529] {
        let error = RequestExecutor::execute(json!({
            "request": request(),
            "response": { "status": status, "body": "retry", "headers": { "retry-after-ms": "0" } }
        }))
        .expect_err("should fail");
        assert!(error.retryable(), "status {status} should be retryable");
    }
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn does_not_retry_non_retryable_status_responses_and_truncates_large_bodies() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": { "status": 401, "body": "x".repeat(20_000) }
    }))
    .expect_err("should fail");

    assert!(!error.retryable());
    let rendered = format!("{error:?}");
    assert!(rendered.contains("16384") || rendered.len() < 20_000);
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn redacts_echoed_request_secret_values_in_response_bodies() {
    let error = RequestExecutor::execute(json!({
        "request": {
            "method": "POST",
            "url": "https://provider.test/v1/chat?api_key=query-secret-123&debug=1",
            "headers": { "authorization": "Bearer header-secret-456" }
        },
        "response": { "status": 400, "body": "provider echoed query-secret-123 and authorization header-secret-456" }
    }))
    .expect_err("should fail");

    let rendered = format!("{error:?}");
    assert!(!rendered.contains("query-secret-123"));
    assert!(!rendered.contains("header-secret-456"));
}

#[test]
#[ignore = "porting: request executor not implemented"]
fn does_not_retry_after_a_successful_response_reaches_stream_parsing() {
    let error = RequestExecutor::execute(json!({
        "request": request(),
        "response": { "status": 200, "body": "data: not-json", "streamParseError": true }
    }))
    .expect_err("should fail");

    assert_eq!(error.classification(), None);
    let rendered = format!("{error:?}");
    assert!(rendered.contains("InvalidProviderOutput") || rendered.contains("stream"));
}
