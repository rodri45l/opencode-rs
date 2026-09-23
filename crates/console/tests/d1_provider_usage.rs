//! Port of packages/console/app/test/providerUsage.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Zen provider helpers: SystemOne request preparation
//! (URL suffix + auth/session headers), Google usage metadata normalization
//! (prompt minus cached, candidates plus thoughts), OpenAI Responses nested
//! usage, SystemOne usage, and OpenAI stream cache-write accounting with
//! clamping when detail fields overlap.
//! Re-derived: JSON payloads are represented as typed usage values; the stream
//! parsers take the raw SSE text.

use opencode_console::provider_usage::{
    google_normalize_usage, google_parse_stream_usage, openai_extract_usage,
    openai_parse_stream_usage, systemone_extract_usage, systemone_modify_headers,
    systemone_modify_url, GoogleUsageMetadata, NormalizedUsage,
};

#[test]
fn prepares_systemone_requests() {
    let mut headers = std::collections::BTreeMap::new();
    systemone_modify_headers(&mut headers, "secret", "session");

    assert_eq!(
        systemone_modify_url("https://api.typesafe.ai/v1/"),
        "https://api.typesafe.ai/v1/systemone"
    );
    assert_eq!(
        headers.get("authorization").map(String::as_str),
        Some("Bearer secret")
    );
    assert_eq!(
        headers.get("x-session-affinity").map(String::as_str),
        Some("session")
    );
}

#[test]
fn extracts_google_non_stream_usage_metadata() {
    let usage = google_normalize_usage(GoogleUsageMetadata {
        prompt_token_count: 10,
        candidates_token_count: 3,
        thoughts_token_count: Some(2),
        cached_content_token_count: Some(4),
    });

    assert_eq!(
        usage,
        NormalizedUsage {
            input_tokens: 6,
            output_tokens: 5,
            reasoning_tokens: Some(2),
            cache_read_tokens: Some(4),
            cache_write_5m_tokens: None,
            cache_write_1h_tokens: None,
        }
    );
}

#[test]
fn parses_google_stream_usage_metadata() {
    let usage = google_parse_stream_usage(
        "data: {\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":3,\"thoughtsTokenCount\":2,\"cachedContentTokenCount\":4}}",
    );

    assert_eq!(
        usage,
        NormalizedUsage {
            input_tokens: 6,
            output_tokens: 5,
            reasoning_tokens: Some(2),
            cache_read_tokens: Some(4),
            cache_write_5m_tokens: None,
            cache_write_1h_tokens: None,
        }
    );
}

#[test]
fn extracts_nested_openai_responses_usage() {
    assert_eq!(
        openai_extract_usage(5, 7),
        NormalizedUsage {
            input_tokens: 5,
            output_tokens: 7,
            reasoning_tokens: None,
            cache_read_tokens: None,
            cache_write_5m_tokens: None,
            cache_write_1h_tokens: None,
        }
    );
}

#[test]
fn extracts_systemone_usage() {
    assert_eq!(
        systemone_extract_usage(312, 48),
        NormalizedUsage {
            input_tokens: 312,
            output_tokens: 48,
            reasoning_tokens: None,
            cache_read_tokens: None,
            cache_write_5m_tokens: None,
            cache_write_1h_tokens: None,
        }
    );
}

#[test]
fn parses_openai_stream_cache_write_usage() {
    let usage = openai_parse_stream_usage(
        "event: response.completed\ndata: {\"response\":{\"usage\":{\"input_tokens\":10,\"input_tokens_details\":{\"cached_tokens\":4,\"cache_write_tokens\":3},\"output_tokens\":2}}}",
    );

    assert_eq!(
        usage,
        NormalizedUsage {
            input_tokens: 3,
            output_tokens: 2,
            reasoning_tokens: None,
            cache_read_tokens: Some(4),
            cache_write_5m_tokens: Some(3),
            cache_write_1h_tokens: None,
        }
    );
}

#[test]
fn clamps_input_tokens_when_detail_fields_overlap() {
    let usage = openai_parse_stream_usage(
        "event: response.completed\ndata: {\"response\":{\"usage\":{\"input_tokens\":5,\"input_tokens_details\":{\"cached_tokens\":4,\"cache_write_tokens\":3},\"output_tokens\":2}}}",
    );

    assert_eq!(
        usage,
        NormalizedUsage {
            input_tokens: 0,
            output_tokens: 2,
            reasoning_tokens: None,
            cache_read_tokens: Some(4),
            cache_write_5m_tokens: Some(3),
            cache_write_1h_tokens: None,
        }
    );
}
