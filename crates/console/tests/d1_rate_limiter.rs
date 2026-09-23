//! Port of packages/console/app/test/rateLimiter.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/routes/zen/util/ipRateLimiter.ts:
//! `getRetryAfterDay` returns the seconds remaining until the next UTC day,
//! rounded up to the nearest second.
//! Re-derived: the timestamp is an explicit UTC time-of-day value.

use opencode_console::rate_limiter::{get_retry_after_day, UtcTime};

fn utc(hour: u32, minute: u32, second: u32, millis: u32) -> UtcTime {
    UtcTime {
        hour,
        minute,
        second,
        millis,
    }
}

#[test]
fn returns_full_day_at_midnight_utc() {
    assert_eq!(get_retry_after_day(utc(0, 0, 0, 0)), 86_400);
}

#[test]
fn returns_remaining_seconds_until_next_utc_day() {
    assert_eq!(get_retry_after_day(utc(12, 0, 0, 0)), 43_200);
}

#[test]
fn rounds_up_to_nearest_second() {
    assert_eq!(get_retry_after_day(utc(23, 59, 59, 500)), 1);
}
