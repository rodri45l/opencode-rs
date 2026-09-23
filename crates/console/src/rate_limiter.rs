//! UTC IP rate-limit retry window.
//!
//! Port of `packages/console/app/src/routes/zen/util/ipRateLimiter.ts`
//! (upstream 18ef3cc): seconds remaining until the next UTC day, rounded up.

/// A UTC time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millis: u32,
}

/// Seconds until the next UTC midnight, rounded up.
pub fn get_retry_after_day(now: UtcTime) -> i64 {
    let within = now.hour as i64 * 3_600_000
        + now.minute as i64 * 60_000
        + now.second as i64 * 1_000
        + now.millis as i64;
    let remaining = 86_400_000 - within;
    (remaining + 999) / 1_000
}
