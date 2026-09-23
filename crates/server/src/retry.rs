//! Session retry delay and retryability classification.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/retry.ts`:
//! exponential backoff capped at 30s with optional jitter, `retry-after`
//! headers honoured ahead of the exponential delay, and a classifier that maps
//! provider errors to retry decisions (including the Go upsell actions).

use serde_json::Value;

/// Maximum number of retries before the loop gives up.
pub const RETRY_MAX_RETRIES: usize = 5;
/// The runtime timer ceiling for a single delay.
pub const RETRY_MAX_DELAY: u64 = 2_147_483_647;
/// Message shown when the free tier limit is reached.
pub const GO_UPSELL_MESSAGE: &str =
    "Subscribe to OpenCode Go for reliable access to the best open-source models for $10/month.";
/// Link for the Go upsell action.
pub const GO_UPSELL_URL: &str = "https://opencode.ai/go";

const BASE_DELAY_MS: u64 = 1000;
const MAX_EXPONENTIAL_MS: u64 = 30_000;
const JITTER_STEP: f64 = 0.25;

/// The error shape the retry classifier consumes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RetryError {
    /// Human message.
    pub message: String,
    /// Whether the provider flagged the error as retryable.
    pub is_retryable: bool,
    /// HTTP status code, when known.
    pub status_code: Option<u16>,
    /// Response headers.
    pub response_headers: Vec<(String, String)>,
    /// Raw response body, when known.
    pub response_body: Option<String>,
    /// Extra metadata.
    pub metadata: Value,
}

impl RetryError {
    /// Construct an error from a message.
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Self::default()
        }
    }

    /// Construct a retryable error with headers.
    pub fn with_headers(message: impl Into<String>, headers: Vec<(String, String)>) -> Self {
        Self {
            message: message.into(),
            is_retryable: true,
            response_headers: headers,
            ..Self::default()
        }
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.response_headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// A retry decision with an optional actionable upsell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryInfo {
    /// The message shown to the user.
    pub message: String,
    /// Optional actionable upsell.
    pub action: Option<RetryAction>,
}

/// An actionable retry upsell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryAction {
    /// Machine-readable reason.
    pub reason: String,
    /// Provider id.
    pub provider: String,
    /// Short title.
    pub title: String,
    /// Body message.
    pub message: String,
    /// Button label.
    pub label: String,
    /// Target link.
    pub link: String,
}

/// Compute the delay before retry `attempt` (1-based).
pub fn delay(attempt: usize, error: &RetryError, jitter: u32) -> u64 {
    if let Some(ms) = error
        .header("retry-after-ms")
        .and_then(|value| value.trim().parse::<u64>().ok())
    {
        return ms.min(RETRY_MAX_DELAY);
    }
    if let Some(value) = error.header("retry-after") {
        if let Ok(seconds) = value.trim().parse::<u64>() {
            return seconds.saturating_mul(1000).min(RETRY_MAX_DELAY);
        }
        if let Some(ms) = parse_http_date_ms(value.trim()) {
            return ms.min(RETRY_MAX_DELAY);
        }
    }

    let exponential = (BASE_DELAY_MS << attempt.min(40)).min(MAX_EXPONENTIAL_MS);
    let jittered = (exponential as f64 * (1.0 + JITTER_STEP * f64::from(jitter))) as u64;
    jittered.min(MAX_EXPONENTIAL_MS)
}

/// Classify an error as retryable, returning the message (and upsell) to show.
pub fn retryable(error: &RetryError, provider: &str) -> Option<RetryInfo> {
    if is_context_overflow(error) {
        return None;
    }

    if error.is_api_error() {
        let status = error.status_code;
        let retryable_message = matches_retryable(&error.message)
            || error
                .response_body
                .as_deref()
                .is_some_and(matches_retryable);
        if !error.is_retryable && !status.is_some_and(|code| code >= 500) && !retryable_message {
            return None;
        }

        let body = error.response_body.as_deref().unwrap_or_default();
        if body.contains("FreeUsageLimitError") {
            return Some(RetryInfo {
                message: GO_UPSELL_MESSAGE.to_string(),
                action: Some(RetryAction {
                    reason: "free_tier_limit".to_string(),
                    provider: provider.to_string(),
                    title: "Free limit reached".to_string(),
                    message: "Subscribe to OpenCode Go for reliable access to the best open-source models for $10/month.".to_string(),
                    label: "subscribe".to_string(),
                    link: GO_UPSELL_URL.to_string(),
                }),
            });
        }
        if body.contains("GoUsageLimitError") {
            let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
            let metadata = parsed.get("metadata");
            let workspace = metadata
                .and_then(|value| value.get("workspace"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let limit_name = metadata
                .and_then(|value| value.get("limitName"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let retry_after = error
                .header("retry-after")
                .and_then(|value| value.trim().parse::<f64>().ok());
            let reset_in = reset_in_text(retry_after);
            let message = format!(
                "{} reached. It will reset in {}. To continue using this model now, enable usage from your available balance",
                if limit_name.is_empty() {
                    "Usage limit".to_string()
                } else {
                    format!("{limit_name} usage limit")
                },
                reset_in
            );
            let link = format!("https://opencode.ai/workspace/{workspace}/go");
            return Some(RetryInfo {
                message: format!("{message} - {link}"),
                action: Some(RetryAction {
                    reason: "account_rate_limit".to_string(),
                    provider: provider.to_string(),
                    title: "Go limit reached".to_string(),
                    message,
                    label: "open settings".to_string(),
                    link,
                }),
            });
        }

        return Some(RetryInfo {
            message: if error.message.contains("Overloaded") {
                "Provider is overloaded".to_string()
            } else {
                error.message.clone()
            },
            action: None,
        });
    }

    let message = error.message.as_str();
    let lower = message.to_lowercase();
    if lower.contains("too_many_requests") {
        return Some(RetryInfo {
            message: "Too Many Requests".to_string(),
            action: None,
        });
    }
    if lower.contains("exhausted") || lower.contains("unavailable") {
        return Some(RetryInfo {
            message: "Provider is overloaded".to_string(),
            action: None,
        });
    }
    if matches_retryable(message) {
        return Some(RetryInfo {
            message: message.to_string(),
            action: None,
        });
    }
    None
}

impl RetryError {
    fn is_api_error(&self) -> bool {
        self.is_retryable
            || self.status_code.is_some()
            || self.response_body.is_some()
            || !self.response_headers.is_empty()
    }
}

fn is_context_overflow(error: &RetryError) -> bool {
    error.message.contains("exceeds context window")
        || error
            .response_body
            .as_deref()
            .is_some_and(|body| body.contains("context_length_exceeded"))
}

fn reset_in_text(retry_after: Option<f64>) -> String {
    let Some(seconds) = retry_after else {
        return String::new();
    };
    let seconds = seconds.max(0.0).ceil() as u64;
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = ((seconds % 3_600) as f64 / 60.0).ceil() as u64;
    let unit = |value: u64, name: &str| {
        if value == 1 {
            format!("{value} {name}")
        } else {
            format!("{value} {name}s")
        }
    };
    if days > 0 {
        if hours > 0 {
            format!("{} {}", unit(days, "day"), unit(hours, "hour"))
        } else {
            unit(days, "day")
        }
    } else if hours > 0 {
        if minutes > 0 {
            format!("{} {}", unit(hours, "hour"), unit(minutes, "minute"))
        } else {
            unit(hours, "hour")
        }
    } else if minutes > 0 {
        unit(minutes, "minute")
    } else {
        "less than a minute".to_string()
    }
}

fn matches_retryable(value: &str) -> bool {
    retry_patterns()
        .iter()
        .any(|pattern| pattern.is_match(value))
}

fn retry_patterns() -> &'static [regex::Regex] {
    use std::sync::OnceLock;
    static PATTERNS: OnceLock<Vec<regex::Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        const SOURCES: &[&str] = &[
            r"429|500|502|503|504|524",
            r"rate increased too quickly|rate limit|rate-limit|rate_limit|too many requests",
            r"overloaded|service unavailable|service_unavailable|service-unavailable|internal error|internal_error|internal server error|server error|server_error|server-error|provider returned error|provider_returned_error|provider-returned-error",
            r"terminated|fetch failed|failed to fetch|network[-_\s]error|upstream connect|connection error|connection refused|connection lost|socket connection was closed|socket hang up|reset before headers|getaddrinfo|enotfound|eai_again|econnrefused|econnreset|etimedout",
            r"^timeout$|\b(?:request|response|connection|network|stream|read) (?:timeout|timed out|time out)\b",
            r"try your request again|retry your request|resource exhausted|resource_exhausted",
            r"\btry again (?:later|in\b)|\b(?:currently|temporarily) at capacity\b",
        ];
        SOURCES
            .iter()
            .map(|source| regex::Regex::new(&format!("(?i){source}")).expect("valid pattern"))
            .collect()
    })
}

/// Parse an HTTP-date and return the positive millisecond delta from now.
fn parse_http_date_ms(value: &str) -> Option<u64> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    let day: i64 = parts[1].parse().ok()?;
    let month = month_number(parts[2])?;
    let year: i64 = parts[3].parse().ok()?;
    let time: Vec<&str> = parts[4].split(':').collect();
    if time.len() != 3 {
        return None;
    }
    let hour: i64 = time[0].parse().ok()?;
    let minute: i64 = time[1].parse().ok()?;
    let second: i64 = time[2].parse().ok()?;

    let target = days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let delta = (target - now) * 1000;
    if delta <= 0 {
        None
    } else {
        Some(delta as u64)
    }
}

fn month_number(name: &str) -> Option<i64> {
    match name {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
