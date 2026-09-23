//! Port of packages/opencode/test/session/retry.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: exponential backoff with jitter, `retry-after` header
//! precedence, and the retryability classifier. The Effect `Schedule` policy
//! cases, the `MessageV2.fromError` transport cases, and the live ECONNRESET
//! server case are dropped (no Effect schedule / fromError surface yet).

use opencode_server::retry::{delay, retryable, RetryError, RETRY_MAX_DELAY};

fn api_error(headers: Vec<(&str, &str)>) -> RetryError {
    RetryError::with_headers(
        "boom",
        headers
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
    )
}

fn wrap(message: &str) -> RetryError {
    RetryError::message(message)
}

fn message_of(error: &RetryError, provider: &str) -> Option<String> {
    retryable(error, provider).map(|info| info.message)
}

#[test]
fn caps_delay_at_30_seconds_when_headers_missing() {
    let error = api_error(vec![]);
    let delays: Vec<u64> = (1..=10).map(|attempt| delay(attempt, &error, 0)).collect();
    assert_eq!(
        delays,
        vec![2000, 4000, 8000, 16000, 30000, 30000, 30000, 30000, 30000, 30000]
    );
}

#[test]
fn adds_jitter_to_exponential_delays() {
    let error = api_error(vec![]);
    assert_eq!(delay(1, &error, 0), 2000);
    assert_eq!(delay(1, &error, 1), 2500);
    assert_eq!(delay(4, &error, 1), 20000);
    assert_eq!(delay(5, &error, 1), 30000);
}

#[test]
fn prefers_retry_after_ms_when_shorter_than_exponential() {
    let error = api_error(vec![("retry-after-ms", "1500")]);
    assert_eq!(delay(4, &error, 0), 1500);
}

#[test]
fn uses_retry_after_seconds_when_reasonable() {
    let error = api_error(vec![("retry-after", "30")]);
    assert_eq!(delay(3, &error, 0), 30000);
}

#[test]
fn accepts_http_date_retry_after_values() {
    let date = httpdate_after_millis(20_000);
    let error = api_error(vec![("retry-after", &date)]);
    let d = delay(1, &error, 0);
    assert!((19_000..=20_000).contains(&d), "unexpected delay: {d}");
}

#[test]
fn ignores_invalid_retry_hints() {
    let error = api_error(vec![("retry-after", "not-a-number")]);
    assert_eq!(delay(1, &error, 0), 2000);
}

#[test]
fn ignores_malformed_date_retry_hints() {
    let error = api_error(vec![("retry-after", "Invalid Date String")]);
    assert_eq!(delay(1, &error, 0), 2000);
}

#[test]
fn ignores_past_date_retry_hints() {
    let date = httpdate_after_millis(-5_000);
    let error = api_error(vec![("retry-after", &date)]);
    assert_eq!(delay(1, &error, 0), 2000);
}

#[test]
fn uses_retry_after_values_even_when_exceeding_10_minutes_with_headers() {
    let error = api_error(vec![("retry-after", "50")]);
    assert_eq!(delay(1, &error, 0), 50000);

    let long_error = api_error(vec![("retry-after-ms", "700000")]);
    assert_eq!(delay(1, &long_error, 0), 700000);
}

#[test]
fn caps_oversized_header_delays_to_the_runtime_timer_limit() {
    let error = api_error(vec![("retry-after-ms", "999999999999")]);
    assert_eq!(delay(1, &error, 0), RETRY_MAX_DELAY);
}

#[test]
fn retries_serialized_too_many_requests_messages() {
    let error = wrap(
        &serde_json::json!({ "type": "error", "error": { "type": "too_many_requests" } })
            .to_string(),
    );
    assert_eq!(
        message_of(&error, "test"),
        Some("Too Many Requests".to_string())
    );
}

#[test]
fn retries_serialized_overloaded_provider_codes() {
    let error = wrap(&serde_json::json!({ "code": "resource_exhausted" }).to_string());
    assert_eq!(
        message_of(&error, "test"),
        Some("Provider is overloaded".to_string())
    );
}

#[test]
fn retries_serialized_rate_limit_messages() {
    let message =
        serde_json::json!({ "type": "error", "error": { "code": "rate_limit_exceeded" } })
            .to_string();
    assert_eq!(message_of(&wrap(&message), "test"), Some(message));
}

#[test]
fn does_not_retry_unknown_json_messages() {
    let error = wrap(&serde_json::json!({ "error": { "message": "no_kv_space" } }).to_string());
    assert_eq!(message_of(&error, "test"), None);
}

#[test]
fn does_not_throw_on_numeric_error_codes() {
    let error = wrap(&serde_json::json!({ "type": "error", "error": { "code": 123 } }).to_string());
    assert_eq!(message_of(&error, "test"), None);
}

#[test]
fn returns_none_for_non_json_message() {
    assert_eq!(message_of(&wrap("not-json"), "test"), None);
}

#[test]
fn retries_plain_text_rate_limit_errors_from_alibaba() {
    let message = "Upstream error from Alibaba: Request rate increased too quickly. To ensure system stability, please adjust your client logic to scale requests more smoothly over time.";
    assert_eq!(
        message_of(&wrap(message), "test"),
        Some(message.to_string())
    );
}

#[test]
fn retries_plain_text_rate_limit_errors() {
    let message = "Rate limit exceeded, please try again later";
    assert_eq!(
        message_of(&wrap(message), "test"),
        Some(message.to_string())
    );
}

#[test]
fn retries_too_many_requests_in_plain_text() {
    let message = "Too many requests, please slow down";
    assert_eq!(
        message_of(&wrap(message), "test"),
        Some(message.to_string())
    );
}

#[test]
fn retries_matching_api_error_text() {
    let cases = [
        "Internal server error",
        "internal error",
        "server-error",
        "Provider returned error",
        "provider-returned-error",
        "terminated",
        "fetch failed",
        "network error",
        "network-error",
        "network_error",
        "connection refused",
        "connect ECONNREFUSED",
        "request ETIMEDOUT",
        "failed to fetch",
        "EAI_AGAIN",
        "response timed out",
        "Please retry your request",
        "try your request again",
        "Please try again in a few minutes",
        "The model is currently at capacity due to high demand",
        "The service is temporarily at capacity",
        "upstream returned status 524",
    ];
    for message in cases {
        assert_eq!(
            message_of(&wrap(message), "test"),
            Some(message.to_string()),
            "expected retryable: {message}"
        );
    }
}

#[test]
fn retries_hyphenated_service_unavailable_errors() {
    assert_eq!(
        message_of(&wrap("service-unavailable"), "test"),
        Some("Provider is overloaded".to_string())
    );
}

#[test]
fn matches_retryable_api_response_bodies() {
    let error = RetryError {
        message: "Request failed".to_string(),
        status_code: Some(400),
        response_body: Some(
            serde_json::json!({ "error": { "message": "upstream connection refused" } })
                .to_string(),
        ),
        ..RetryError::default()
    };
    assert_eq!(
        message_of(&error, "test"),
        Some("Request failed".to_string())
    );
}

#[test]
fn does_not_retry_context_overflow_errors() {
    let error = RetryError {
        message: "Input exceeds context window of this model".to_string(),
        response_body: Some(
            serde_json::json!({ "error": { "code": "context_length_exceeded" } }).to_string(),
        ),
        ..RetryError::default()
    };
    assert_eq!(message_of(&error, "test"), None);
}

#[test]
fn retries_500_errors_even_when_is_retryable_is_false() {
    let error = RetryError {
        message: "Internal server error".to_string(),
        status_code: Some(500),
        response_body: Some(
            serde_json::json!({ "type": "api_error", "message": "Internal server error" })
                .to_string(),
        ),
        ..RetryError::default()
    };
    assert_eq!(
        message_of(&error, "test"),
        Some("Internal server error".to_string())
    );
}

#[test]
fn retries_502_and_503_errors() {
    let bad_gateway = RetryError {
        message: "Bad gateway".to_string(),
        status_code: Some(502),
        ..RetryError::default()
    };
    let unavailable = RetryError {
        message: "Service unavailable".to_string(),
        status_code: Some(503),
        ..RetryError::default()
    };
    assert_eq!(
        message_of(&bad_gateway, "test"),
        Some("Bad gateway".to_string())
    );
    assert_eq!(
        message_of(&unavailable, "test"),
        Some("Service unavailable".to_string())
    );
}

#[test]
fn does_not_retry_4xx_errors_when_is_retryable_is_false() {
    let error = RetryError {
        message: "Bad request".to_string(),
        status_code: Some(400),
        ..RetryError::default()
    };
    assert_eq!(message_of(&error, "test"), None);
}

#[test]
fn retries_zlib_decompression_failures() {
    let error = RetryError {
        message: "Response decompression failed".to_string(),
        is_retryable: true,
        metadata: serde_json::json!({ "code": "ZlibError" }),
        ..RetryError::default()
    };
    assert_eq!(
        message_of(&error, "test"),
        Some("Response decompression failed".to_string())
    );
}

#[test]
fn maps_free_limits_to_go_upsell_action() {
    let error = RetryError {
        message: "Free usage exceeded".to_string(),
        is_retryable: true,
        status_code: Some(429),
        response_body: Some(
            serde_json::json!({
                "type": "error",
                "error": { "type": "FreeUsageLimitError", "message": "Free usage exceeded" }
            })
            .to_string(),
        ),
        ..RetryError::default()
    };
    let info = retryable(&error, "opencode").expect("retryable");
    assert_eq!(info.message, opencode_server::retry::GO_UPSELL_MESSAGE);
    let action = info.action.expect("action");
    assert_eq!(action.reason, "free_tier_limit");
    assert_eq!(action.provider, "opencode");
    assert_eq!(action.title, "Free limit reached");
    assert_eq!(action.label, "subscribe");
    assert_eq!(action.link, opencode_server::retry::GO_UPSELL_URL);
}

#[test]
fn maps_go_subscription_limits_to_workspace_payg_upsell() {
    let error = RetryError {
        message: "Subscription quota exceeded. You can continue using free models.".to_string(),
        is_retryable: true,
        status_code: Some(429),
        response_headers: vec![("retry-after".to_string(), "19380".to_string())],
        response_body: Some(
            serde_json::json!({
                "type": "error",
                "error": {
                    "type": "GoUsageLimitError",
                    "message": "Subscription quota exceeded. You can continue using free models."
                },
                "metadata": {
                    "workspace": "wrk_01K6XGM22R6FM8JVABE9XDQXGH",
                    "limitName": "5 hour"
                }
            })
            .to_string(),
        ),
        ..RetryError::default()
    };
    let info = retryable(&error, "opencode-go").expect("retryable");
    assert_eq!(
        info.message,
        "5 hour usage limit reached. It will reset in 5 hours 23 minutes. To continue using this model now, enable usage from your available balance - https://opencode.ai/workspace/wrk_01K6XGM22R6FM8JVABE9XDQXGH/go"
    );
    let action = info.action.expect("action");
    assert_eq!(action.reason, "account_rate_limit");
    assert_eq!(action.provider, "opencode-go");
    assert_eq!(action.label, "open settings");
}

#[test]
fn maps_go_subscription_limits_without_limit_metadata() {
    let error = RetryError {
        message: "Subscription quota exceeded. You can continue using free models.".to_string(),
        is_retryable: true,
        status_code: Some(429),
        response_headers: vec![("retry-after".to_string(), "900".to_string())],
        response_body: Some(
            serde_json::json!({
                "type": "error",
                "error": {
                    "type": "GoUsageLimitError",
                    "message": "Subscription quota exceeded. You can continue using free models."
                },
                "metadata": { "workspace": "wrk_01K6XGM22R6FM8JVABE9XDQXGH" }
            })
            .to_string(),
        ),
        ..RetryError::default()
    };
    assert_eq!(
        retryable(&error, "opencode-go").and_then(|info| info.action).map(|action| action.message),
        Some(
            "Usage limit reached. It will reset in 15 minutes. To continue using this model now, enable usage from your available balance"
                .to_string()
        )
    );
}

fn httpdate_after_millis(offset: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let target = now + offset / 1000;
    let days = target.div_euclid(86_400);
    let seconds = target.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let weekday = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][(days.rem_euclid(7)) as usize];
    let month_name = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][(month - 1) as usize];
    format!(
        "{weekday}, {day:02} {month_name} {year} {:02}:{:02}:{:02} GMT",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}
