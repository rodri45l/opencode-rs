//! Port of packages/opencode/test/provider/error.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: provider stream errors without a code classify as
//! retryable `api_error`s carrying the provider message and the raw body.
//! The code-specific branches are pinned by the retry/error suites.

use opencode_server::port::provider::parse_stream_error;
use serde_json::json;

#[test]
fn retries_provider_stream_errors_without_a_code() {
    let messages = [
        "The model is currently at capacity due to high demand. Please try again in a few minutes, or use a higher service tier for priority processing: https://docs.x.ai/developers/advanced-api-usage/priority-processing",
        "The model is temporarily unavailable.",
    ];

    for message in messages {
        let parsed = parse_stream_error(&json!({
            "type": "error",
            "error": { "message": message },
        }))
        .expect("stream error parses");

        assert_eq!(parsed.kind, "api_error");
        assert_eq!(parsed.message, message);
        assert!(parsed.is_retryable);
        assert_eq!(
            parsed.response_body,
            serde_json::to_string(&json!({
                "type": "error",
                "error": { "message": message },
            }))
            .expect("body serializes")
        );
    }
}
