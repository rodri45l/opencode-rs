//! Zen provider usage extraction.
//!
//! Port of the console Zen provider helpers (upstream 18ef3cc): Google and
//! OpenAI Responses normalization, SystemOne request preparation, and the
//! OpenAI stream cache-write accounting with clamping.

use serde_json::Value;

/// A normalized usage record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_5m_tokens: Option<i64>,
    pub cache_write_1h_tokens: Option<i64>,
}

/// Google `usageMetadata`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoogleUsageMetadata {
    pub prompt_token_count: i64,
    pub candidates_token_count: i64,
    pub thoughts_token_count: Option<i64>,
    pub cached_content_token_count: Option<i64>,
}

/// OpenAI Responses usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenAiUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
}

/// Normalize Google usage metadata.
pub fn google_normalize_usage(metadata: GoogleUsageMetadata) -> NormalizedUsage {
    let reasoning = metadata.thoughts_token_count.unwrap_or(0);
    let cache_read = metadata.cached_content_token_count.unwrap_or(0);
    NormalizedUsage {
        input_tokens: metadata.prompt_token_count - cache_read,
        output_tokens: metadata.candidates_token_count + reasoning,
        reasoning_tokens: Some(reasoning),
        cache_read_tokens: Some(cache_read),
        cache_write_5m_tokens: None,
        cache_write_1h_tokens: None,
    }
}

/// Parse Google stream usage from raw SSE text.
pub fn google_parse_stream_usage(body: &str) -> NormalizedUsage {
    for line in body.lines() {
        let Some(data) = line.strip_prefix("data: ") else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        let Some(usage) = json.get("usageMetadata") else {
            continue;
        };
        let metadata = GoogleUsageMetadata {
            prompt_token_count: usage
                .get("promptTokenCount")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            candidates_token_count: usage
                .get("candidatesTokenCount")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            thoughts_token_count: usage.get("thoughtsTokenCount").and_then(Value::as_i64),
            cached_content_token_count: usage
                .get("cachedContentTokenCount")
                .and_then(Value::as_i64),
        };
        return google_normalize_usage(metadata);
    }
    NormalizedUsage {
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: None,
        cache_read_tokens: None,
        cache_write_5m_tokens: None,
        cache_write_1h_tokens: None,
    }
}

/// Normalize OpenAI Responses usage.
pub fn openai_normalize_usage(usage: OpenAiUsage) -> NormalizedUsage {
    let cache_read = usage.cached_tokens;
    let cache_write = usage.cache_write_tokens;
    NormalizedUsage {
        input_tokens: (usage.input_tokens - cache_read.unwrap_or(0) - cache_write.unwrap_or(0))
            .max(0),
        output_tokens: usage.output_tokens,
        reasoning_tokens: None,
        cache_read_tokens: cache_read,
        cache_write_5m_tokens: cache_write,
        cache_write_1h_tokens: None,
    }
}

/// Parse an OpenAI `response.completed` stream event.
pub fn openai_parse_stream_usage(body: &str) -> NormalizedUsage {
    let lines: Vec<&str> = body.lines().collect();
    for pair in lines.windows(2) {
        if pair[0] != "event: response.completed" {
            continue;
        }
        let Some(data) = pair[1].strip_prefix("data: ") else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        let Some(usage) = json
            .get("response")
            .and_then(|response| response.get("usage"))
        else {
            continue;
        };
        let details = usage.get("input_tokens_details");
        return openai_normalize_usage(OpenAiUsage {
            input_tokens: usage
                .get("input_tokens")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            output_tokens: usage
                .get("output_tokens")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            cached_tokens: details
                .and_then(|details| details.get("cached_tokens"))
                .and_then(Value::as_i64),
            cache_write_tokens: details
                .and_then(|details| details.get("cache_write_tokens"))
                .and_then(Value::as_i64),
        });
    }
    NormalizedUsage {
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: None,
        cache_read_tokens: None,
        cache_write_5m_tokens: None,
        cache_write_1h_tokens: None,
    }
}

/// Build usage from an extracted OpenAI input/output pair.
pub fn openai_extract_usage(input_tokens: i64, output_tokens: i64) -> NormalizedUsage {
    NormalizedUsage {
        input_tokens,
        output_tokens,
        reasoning_tokens: None,
        cache_read_tokens: None,
        cache_write_5m_tokens: None,
        cache_write_1h_tokens: None,
    }
}

/// The SystemOne `/systemone` URL.
pub fn systemone_modify_url(url: &str) -> String {
    format!("{}/systemone", url.trim_end_matches('/'))
}

/// Set the SystemOne auth/session headers.
pub fn systemone_modify_headers(
    headers: &mut std::collections::BTreeMap<String, String>,
    secret: &str,
    session: &str,
) {
    headers.insert("authorization".to_string(), format!("Bearer {secret}"));
    headers.insert("x-session-affinity".to_string(), session.to_string());
}

/// Build usage from an extracted SystemOne input/output pair.
pub fn systemone_extract_usage(input_tokens: i64, output_tokens: i64) -> NormalizedUsage {
    NormalizedUsage {
        input_tokens,
        output_tokens,
        reasoning_tokens: None,
        cache_read_tokens: None,
        cache_write_5m_tokens: None,
        cache_write_1h_tokens: None,
    }
}
