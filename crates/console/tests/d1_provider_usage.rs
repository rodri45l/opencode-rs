//! Port of packages/console/app/test/providerUsage.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the Zen provider helpers: SystemOne request preparation
//! (URL suffix + auth/session headers), Google usage metadata normalization
//! (prompt minus cached, candidates plus thoughts), OpenAI Responses nested
//! usage, SystemOne usage, and OpenAI stream cache-write accounting with
//! clamping when detail fields overlap.
//! Re-derived: JSON payloads are represented as typed usage values; the stream
//! parsers take the raw SSE text.

#[allow(dead_code)]
mod provider_usage {
    use std::collections::BTreeMap;
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: console provider usage extraction not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NormalizedUsage {
        pub input_tokens: i64,
        pub output_tokens: i64,
        pub reasoning_tokens: Option<i64>,
        pub cache_read_tokens: Option<i64>,
        pub cache_write_5m_tokens: Option<i64>,
        pub cache_write_1h_tokens: Option<i64>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GoogleUsageMetadata {
        pub prompt_token_count: i64,
        pub candidates_token_count: i64,
        pub thoughts_token_count: Option<i64>,
        pub cached_content_token_count: Option<i64>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenAiUsage {
        pub input_tokens: i64,
        pub output_tokens: i64,
        pub cached_tokens: Option<i64>,
        pub cache_write_tokens: Option<i64>,
    }

    pub fn google_normalize_usage(_metadata: GoogleUsageMetadata) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }

    pub fn google_parse_stream_usage(_body: &str) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }

    pub fn openai_normalize_usage(_usage: OpenAiUsage) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }

    pub fn openai_parse_stream_usage(_body: &str) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }

    pub fn openai_extract_usage(
        _input_tokens: i64,
        _output_tokens: i64,
    ) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }

    pub fn systemone_modify_url(_url: &str) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }

    pub fn systemone_modify_headers(
        _headers: &mut BTreeMap<String, String>,
        _secret: &str,
        _session: &str,
    ) -> PortResult<()> {
        Err(NotImplemented(NOTE))
    }

    pub fn systemone_extract_usage(
        _input_tokens: i64,
        _output_tokens: i64,
    ) -> PortResult<NormalizedUsage> {
        Err(NotImplemented(NOTE))
    }
}

use provider_usage::{
    google_normalize_usage, google_parse_stream_usage, openai_extract_usage,
    openai_parse_stream_usage, systemone_extract_usage, systemone_modify_headers,
    systemone_modify_url, GoogleUsageMetadata, NormalizedUsage, NOTE,
};

#[test]
#[ignore = "porting: console provider usage extraction not implemented"]
fn prepares_systemone_requests() {
    let mut headers = std::collections::BTreeMap::new();
    systemone_modify_headers(&mut headers, "secret", "session").expect(NOTE);

    assert_eq!(
        systemone_modify_url("https://api.typesafe.ai/v1/").expect(NOTE),
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
#[ignore = "porting: console provider usage extraction not implemented"]
fn extracts_google_non_stream_usage_metadata() {
    let usage = google_normalize_usage(GoogleUsageMetadata {
        prompt_token_count: 10,
        candidates_token_count: 3,
        thoughts_token_count: Some(2),
        cached_content_token_count: Some(4),
    })
    .expect(NOTE);

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
#[ignore = "porting: console provider usage extraction not implemented"]
fn parses_google_stream_usage_metadata() {
    let usage = google_parse_stream_usage(
        "data: {\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":3,\"thoughtsTokenCount\":2,\"cachedContentTokenCount\":4}}",
    )
    .expect(NOTE);

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
#[ignore = "porting: console provider usage extraction not implemented"]
fn extracts_nested_openai_responses_usage() {
    assert_eq!(
        openai_extract_usage(5, 7).expect(NOTE),
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
#[ignore = "porting: console provider usage extraction not implemented"]
fn extracts_systemone_usage() {
    assert_eq!(
        systemone_extract_usage(312, 48).expect(NOTE),
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
#[ignore = "porting: console provider usage extraction not implemented"]
fn parses_openai_stream_cache_write_usage() {
    let usage = openai_parse_stream_usage(
        "event: response.completed\ndata: {\"response\":{\"usage\":{\"input_tokens\":10,\"input_tokens_details\":{\"cached_tokens\":4,\"cache_write_tokens\":3},\"output_tokens\":2}}}",
    )
    .expect(NOTE);

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
#[ignore = "porting: console provider usage extraction not implemented"]
fn clamps_input_tokens_when_detail_fields_overlap() {
    let usage = openai_parse_stream_usage(
        "event: response.completed\ndata: {\"response\":{\"usage\":{\"input_tokens\":5,\"input_tokens_details\":{\"cached_tokens\":4,\"cache_write_tokens\":3},\"output_tokens\":2}}}",
    )
    .expect(NOTE);

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
