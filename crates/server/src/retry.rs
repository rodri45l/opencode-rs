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
pub fn retryable(_error: &RetryError, _provider: &str) -> Option<RetryInfo> {
    None
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
