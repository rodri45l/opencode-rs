//! Port of packages/console/app/test/rateLimiter.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/routes/zen/util/ipRateLimiter.ts:
//! `getRetryAfterDay` returns the seconds remaining until the next UTC day,
//! rounded up to the nearest second.
//! Re-derived: the timestamp is an explicit UTC time-of-day value.

#[allow(dead_code)]
mod rate_limiter {
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

    pub const NOTE: &str = "porting: console ip rate limiter not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UtcTime {
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
        pub millis: u32,
    }

    pub fn get_retry_after_day(_now: UtcTime) -> PortResult<i64> {
        Err(NotImplemented(NOTE))
    }
}

use rate_limiter::{get_retry_after_day, UtcTime, NOTE};

fn utc(hour: u32, minute: u32, second: u32, millis: u32) -> UtcTime {
    UtcTime {
        hour,
        minute,
        second,
        millis,
    }
}

#[test]
#[ignore = "porting: console ip rate limiter not implemented"]
fn returns_full_day_at_midnight_utc() {
    assert_eq!(get_retry_after_day(utc(0, 0, 0, 0)).expect(NOTE), 86_400);
}

#[test]
#[ignore = "porting: console ip rate limiter not implemented"]
fn returns_remaining_seconds_until_next_utc_day() {
    assert_eq!(get_retry_after_day(utc(12, 0, 0, 0)).expect(NOTE), 43_200);
}

#[test]
#[ignore = "porting: console ip rate limiter not implemented"]
fn rounds_up_to_nearest_second() {
    assert_eq!(get_retry_after_day(utc(23, 59, 59, 500)).expect(NOTE), 1);
}
